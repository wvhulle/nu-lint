use nu_protocol::Span;

use crate::{
    LintLevel,
    ast::string::StringFormat,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'http get', 'http post', etc. for HTTP requests. Nushell's http commands \
                    return structured data and integrate well with pipelines.";

#[derive(Default)]
struct HttpOptions {
    method: HttpMethod,
    url: Option<String>,
    headers: Vec<(String, String)>,
    user: Option<String>,
    password: Option<String>,
    data: Option<DataArg>,
    output_file: Option<String>,
}

enum DataArg {
    Formatted(StringFormat),
    Raw(String),
}

impl HttpOptions {
    fn parse_curl<'a>(args: impl IntoIterator<Item = (&'a str, Option<StringFormat>)>) -> Self {
        args.into_iter()
            .fold(
                (Self::default(), None::<&str>),
                |(mut opts, expecting), (text, format)| match expecting {
                    Some("-X" | "--request") => {
                        opts.method = parse_method(text);
                        (opts, None)
                    }
                    Some("-H" | "--header") => {
                        if let Some(h) = parse_header(text) {
                            opts.headers.push(h);
                        }
                        (opts, None)
                    }
                    Some("-u" | "--user") => {
                        let (user, password) = parse_credentials(text);
                        opts.user = user;
                        opts.password = password;
                        (opts, None)
                    }
                    Some("-d" | "--data" | "--data-raw") => {
                        opts.data = Some(
                            format
                                .map_or_else(|| DataArg::Raw(text.to_string()), DataArg::Formatted),
                        );
                        opts.method = match opts.method {
                            HttpMethod::Get => HttpMethod::Post,
                            m => m,
                        };
                        (opts, None)
                    }
                    Some("-o" | "--output") => {
                        opts.output_file = Some(text.to_string());
                        (opts, None)
                    }
                    None if matches!(
                        text,
                        "-X" | "--request"
                            | "-H"
                            | "--header"
                            | "-u"
                            | "--user"
                            | "-d"
                            | "--data"
                            | "--data-raw"
                            | "-o"
                            | "--output"
                    ) =>
                    {
                        (opts, Some(text))
                    }
                    None if !text.starts_with('-') && opts.url.is_none() => {
                        opts.url = Some(text.to_string());
                        (opts, None)
                    }
                    _ => (opts, None),
                },
            )
            .0
    }

    fn to_nushell(&self) -> (String, String) {
        let url = self.url.as_deref().unwrap_or("URL");

        let mut parts = Vec::new();
        let method_cmd = match self.method {
            HttpMethod::Get => "http get",
            HttpMethod::Post => "http post",
            HttpMethod::Put => "http put",
            HttpMethod::Patch => "http patch",
            HttpMethod::Delete => "http delete",
        };

        parts.push(method_cmd.to_string());

        if let Some(user) = &self.user {
            parts.push(format!("--user {user}"));
        }

        if let Some(password) = &self.password {
            parts.push(format!("--password {password}"));
        }

        if !self.headers.is_empty() {
            let headers_list: Vec<String> = self
                .headers
                .iter()
                .flat_map(|(k, v)| vec![k.clone(), v.clone()])
                .collect();
            parts.push(format!("--headers [{}]", headers_list.join(" ")));
        }

        parts.push(url.to_string());

        if let Some(data) = &self.data {
            let data_str = match data {
                DataArg::Formatted(fmt) => fmt.reconstruct(fmt.content()),
                DataArg::Raw(text) => text.clone(),
            };
            parts.push(data_str);
        }

        if let Some(file) = &self.output_file {
            parts.push(format!("| save {file}"));
        }

        let description = self.build_description();

        (parts.join(" "), description)
    }

    fn build_description(&self) -> String {
        let mut parts = Vec::new();

        parts.push("Replace curl with Nushell's http commands.".to_string());
        parts.push(format!(
            "Method: {} → {}",
            match self.method {
                HttpMethod::Get => "GET",
                HttpMethod::Post => "POST",
                HttpMethod::Put => "PUT",
                HttpMethod::Patch => "PATCH",
                HttpMethod::Delete => "DELETE",
            },
            match self.method {
                HttpMethod::Get => "http get",
                HttpMethod::Post => "http post",
                HttpMethod::Put => "http put",
                HttpMethod::Patch => "http patch",
                HttpMethod::Delete => "http delete",
            }
        ));

        if !self.headers.is_empty() {
            parts.push(format!(
                "Headers: -H flag → --headers [key value ...] ({} headers)",
                self.headers.len()
            ));
        }

        if self.user.is_some() || self.password.is_some() {
            parts.push("Auth: -u user:pass → --user user --password pass".to_string());
        }

        if self.data.is_some() {
            parts.push("Data: -d data → positional argument or structured data".to_string());
        }

        parts.push(
            "Benefits: structured data output, better pipeline integration, consistent API across \
             all HTTP methods."
                .to_string(),
        );

        parts.join(" ")
    }
}

#[derive(Default, PartialEq)]
enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

fn parse_method(method_str: &str) -> HttpMethod {
    match method_str.to_uppercase().as_str() {
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "PATCH" => HttpMethod::Patch,
        "DELETE" => HttpMethod::Delete,
        _ => HttpMethod::Get,
    }
}

fn parse_credentials(credentials: &str) -> (Option<String>, Option<String>) {
    credentials
        .split_once(':')
        .map_or((Some(credentials.to_string()), None), |(user, pass)| {
            (Some(user.to_string()), Some(pass.to_string()))
        })
}

fn parse_header(header: &str) -> Option<(String, String)> {
    header
        .split_once(':')
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
}

struct CurlFixData {
    expr_span: Span,
    options: HttpOptions,
}

struct UseBuiltinCurl;

impl DetectFix for UseBuiltinCurl {
    type FixInput<'a> = CurlFixData;

    fn id(&self) -> &'static str {
        "curl_to_http"
    }

    fn short_description(&self) -> &'static str {
        "`curl` replaceable with `http` commands"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/http_get.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .detect_external_with_validation("curl", |_, fix_data, ctx| {
                // Don't detect very complex curl usage
                let has_complex = fix_data.arg_texts(ctx).any(|text| {
                    matches!(
                        text,
                        "--proxy" | "--socks" |       // Proxy settings
                        "--cert" | "--key" |          // Client certificates
                        "--cacert" | "--capath" |     // CA certificates
                        "--ftp-" |                     // FTP-specific options
                        "--tftp-" |                    // TFTP options
                        "--telnet-" |                  // Telnet options
                        "--resolve" |                  // Custom DNS
                        "--connect-timeout" |         // Connection timeout
                        "--max-time" |                 // Max transfer time
                        "--retry" |                    // Retry logic
                        "--limit-rate" |               // Rate limiting
                        "--compressed" |               // Compression
                        "--tr-encoding" |              // Transfer encoding
                        "--negotiate" | "--ntlm" | "--digest" | "--basic" // Auth methods
                    ) || text.starts_with("--proxy")
                        || text.starts_with("--socks")
                });
                if has_complex { None } else { Some(NOTE) }
            })
            .into_iter()
            .map(|(detection, fix_data)| {
                let args_with_formats = fix_data
                    .arg_texts(context)
                    .zip(fix_data.arg_formats(context));
                let options = HttpOptions::parse_curl(args_with_formats);
                (
                    detection,
                    CurlFixData {
                        expr_span: fix_data.expr_span,
                        options,
                    },
                )
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let (replacement, description) = fix_data.options.to_nushell();

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement::new(fix_data.expr_span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinCurl;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
