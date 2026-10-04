use std::iter::once;

use nu_protocol::{Span, ast::Expression};

use crate::{
    LintLevel,
    ast::{
        external::{CliSpec, ExternalInvocation, Flag, FlagValue, ParsedCli, value_text},
        string::quote_nu_string,
    },
    context::LintContext,
    dsl::arguments::{HttpHeader, http_header},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const FAIL: Flag = Flag::switch('f', "fail");
const LOCATION: Flag = Flag::switch('L', "location");
const SILENT: Flag = Flag::switch('s', "silent");
const SHOW_ERROR: Flag = Flag::switch('S', "show-error");
const INSECURE: Flag = Flag::switch('k', "insecure");
const CURL_HEADER: Flag = Flag::switch('H', "header").with_value();
const CURL_OUTPUT: Flag = Flag::switch('o', "output").with_value();
const REQUEST: Flag = Flag::switch('X', "request").with_value();
const MAX_TIME: Flag = Flag::switch('m', "max-time").with_value();

const QUIET: Flag = Flag::switch('q', "quiet");
const WGET_OUTPUT: Flag = Flag::switch('O', "output-document").with_value();
const WGET_HEADER: Flag = Flag::long_switch("header").with_value();

static CURL_SPEC: CliSpec = CliSpec {
    flags: &[
        FAIL,
        LOCATION,
        SILENT,
        SHOW_ERROR,
        INSECURE,
        CURL_HEADER,
        CURL_OUTPUT,
        REQUEST,
        MAX_TIME,
    ],
    numeric_shorthand: None,
};

static WGET_SPEC: CliSpec = CliSpec {
    flags: &[QUIET, WGET_OUTPUT, WGET_HEADER],
    numeric_shorthand: None,
};

const STANDARD_OUTPUT: &str = "-";
const GET_METHOD: &str = "GET";

const NOTE: &str = "'http get' fails on error status codes and follows redirects like 'curl -fL' \
                    or 'wget', and it is available on every platform without an external download \
                    tool.";

pub struct Download<'a> {
    span: Span,
    url: &'a Expression,
    headers: Vec<HttpHeader<'a>>,
    insecure: bool,
    max_time: Option<u32>,
    output: Option<FlagValue<'a>>,
}

impl<'a> Download<'a> {
    fn from_invocation(invocation: &ExternalInvocation<'a>, context: &LintContext) -> Option<Self> {
        if invocation.next_command_name(context) == Some("complete") {
            return None;
        }
        match invocation.name {
            "curl" => Self::from_curl(invocation.span, &invocation.parse(&CURL_SPEC)?),
            _ => Self::from_wget(invocation.span, &invocation.parse(&WGET_SPEC)?),
        }
    }

    fn from_curl(span: Span, parsed: &ParsedCli<'a>) -> Option<Self> {
        let follows_redirects_and_fails = parsed.has(FAIL) && parsed.has(LOCATION);
        let is_get = parsed
            .value(REQUEST)
            .is_none_or(|method| method.literal() == Some(GET_METHOD));
        if !(follows_redirects_and_fails && is_get) {
            return None;
        }
        let max_time = match parsed.value(MAX_TIME) {
            Some(seconds) => Some(seconds.literal()?.parse().ok()?),
            None => None,
        };
        Some(Self {
            span,
            url: single_url(parsed)?,
            headers: headers(parsed.values(CURL_HEADER))?,
            insecure: parsed.has(INSECURE),
            max_time,
            output: parsed.value(CURL_OUTPUT),
        })
    }

    fn from_wget(span: Span, parsed: &ParsedCli<'a>) -> Option<Self> {
        let output = parsed.value(WGET_OUTPUT)?;
        Some(Self {
            span,
            url: single_url(parsed)?,
            headers: headers(parsed.values(WGET_HEADER))?,
            insecure: false,
            max_time: None,
            output: (output.literal() != Some(STANDARD_OUTPUT)).then_some(output),
        })
    }

    fn to_nu(&self, context: &LintContext) -> String {
        let header_list = (!self.headers.is_empty()).then(|| {
            let pairs: Vec<String> = self
                .headers
                .iter()
                .map(|header| {
                    format!(
                        "{} {}",
                        quote_nu_string(header.name),
                        quote_nu_string(header.value)
                    )
                })
                .collect();
            format!("--headers [{}]", pairs.join(" "))
        });
        let insecure = self.insecure.then(|| "--insecure".to_string());
        let timeout = self
            .max_time
            .map(|seconds| format!("--max-time {seconds}sec"));
        let url = value_text(context, self.url).into_owned();
        let words: Vec<String> = once("http get --raw".to_string())
            .chain(header_list)
            .chain(insecure)
            .chain(timeout)
            .chain(once(url))
            .collect();
        let request = words.join(" ");
        match self.output {
            Some(file) => format!("{request} | save --force {}", file.nu_text(context)),
            None => request,
        }
    }
}

const fn single_url<'a>(parsed: &ParsedCli<'a>) -> Option<&'a Expression> {
    match parsed.operands.as_slice() {
        [url] => Some(url),
        _ => None,
    }
}

fn headers<'a>(values: impl Iterator<Item = FlagValue<'a>>) -> Option<Vec<HttpHeader<'a>>> {
    values
        .map(|header| http_header(header.literal()?))
        .collect()
}

struct CurlWgetToHttp;

impl DetectFix for CurlWgetToHttp {
    type FixInput<'a> = Download<'a>;

    fn id(&self) -> &'static str {
        "curl_wget_to_http"
    }

    fn short_description(&self) -> &'static str {
        "`curl -fL` or `wget -O` download replaceable with `http get`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/http_get.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["curl", "wget"])
            .iter()
            .filter_map(|invocation| {
                let download = Download::from_invocation(invocation, context)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label(format!("'{}' downloading a URL", invocation.name));
                Some((detection, download))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, download: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = download.to_nu(context);
        Some(Fix {
            explanation: format!("Download with '{replacement}'").into(),
            replacements: vec![Replacement::new(download.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &CurlWgetToHttp;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
