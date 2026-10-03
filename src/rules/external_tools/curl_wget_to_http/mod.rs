use std::borrow::Cow;

use crate::{
    LintLevel,
    ast::{
        external::{CliSpec, ExternalInvocation, Flag, FlagValue, ParsedCli, value_text},
        string::quote_nu_string,
    },
    context::LintContext,
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

const NOTE: &str = "'http get' fails on error status codes and follows redirects like 'curl -fL' \
                    or 'wget', and it is available on every platform without an external download \
                    tool.";

struct Download<'a> {
    url: Cow<'a, str>,
    headers: Vec<String>,
    insecure: bool,
    max_time: Option<&'a str>,
    output_file: Option<Cow<'a, str>>,
}

impl<'a> Download<'a> {
    fn from_curl(parsed: &ParsedCli<'a>, context: &'a LintContext) -> Option<Self> {
        let follows_redirects_and_fails = parsed.has(FAIL) && parsed.has(LOCATION);
        let is_get = parsed
            .value(REQUEST)
            .is_none_or(|method| method.literal() == Some("GET"));
        if !(follows_redirects_and_fails && is_get) {
            return None;
        }
        let max_time = match parsed.value(MAX_TIME) {
            Some(seconds) => Some(seconds.literal().filter(|s| s.parse::<u32>().is_ok())?),
            None => None,
        };
        Some(Self {
            url: Self::single_url(parsed, context)?,
            headers: Self::headers(parsed.values(CURL_HEADER))?,
            insecure: parsed.has(INSECURE),
            max_time,
            output_file: parsed
                .value(CURL_OUTPUT)
                .map(|file| Self::output_path(file, context)),
        })
    }

    fn from_wget(parsed: &ParsedCli<'a>, context: &'a LintContext) -> Option<Self> {
        let output = parsed.value(WGET_OUTPUT)?;
        let output_file =
            (output.literal() != Some(STANDARD_OUTPUT)).then(|| Self::output_path(output, context));
        Some(Self {
            url: Self::single_url(parsed, context)?,
            headers: Self::headers(parsed.values(WGET_HEADER))?,
            insecure: false,
            max_time: None,
            output_file,
        })
    }

    fn single_url(parsed: &ParsedCli<'a>, context: &'a LintContext) -> Option<Cow<'a, str>> {
        match parsed.operands.as_slice() {
            [url] => Some(value_text(context, url)),
            _ => None,
        }
    }

    fn output_path(file: FlagValue<'a>, context: &'a LintContext) -> Cow<'a, str> {
        match file {
            FlagValue::Inline(text) => Cow::Owned(quote_nu_string(text)),
            FlagValue::Argument(expr) => Cow::Borrowed(context.expr_text(expr)),
        }
    }

    fn headers(values: impl Iterator<Item = FlagValue<'a>>) -> Option<Vec<String>> {
        values
            .map(|header| {
                let (name, value) = header.literal()?.split_once(':')?;
                Some(format!(
                    "{} {}",
                    quote_nu_string(name.trim()),
                    quote_nu_string(value.trim())
                ))
            })
            .collect()
    }

    fn to_nu(&self) -> String {
        let headers =
            (!self.headers.is_empty()).then(|| format!("--headers [{}]", self.headers.join(" ")));
        let insecure = self.insecure.then(|| "--insecure".to_string());
        let timeout = self
            .max_time
            .map(|seconds| format!("--max-time {seconds}sec"));
        let request = ["http get --raw".to_string()]
            .into_iter()
            .chain(headers)
            .chain(insecure)
            .chain(timeout)
            .chain([self.url.to_string()])
            .collect::<Vec<_>>()
            .join(" ");
        match &self.output_file {
            Some(file) => format!("{request} | save --force {file}"),
            None => request,
        }
    }
}

fn download<'a>(
    invocation: &ExternalInvocation<'a>,
    context: &'a LintContext,
) -> Option<Download<'a>> {
    if invocation.next_command_name(context) == Some("complete") {
        return None;
    }
    match invocation.name {
        "curl" => Download::from_curl(&invocation.parse(&CURL_SPEC)?, context),
        _ => Download::from_wget(&invocation.parse(&WGET_SPEC)?, context),
    }
}

struct CurlWgetToHttp;

impl DetectFix for CurlWgetToHttp {
    type FixInput<'a> = Replacement;

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
                let request = download(invocation, context)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label(format!("'{}' downloading a URL", invocation.name));
                Some((
                    detection,
                    Replacement::new(invocation.span, request.to_nu()),
                ))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Download with '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
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
