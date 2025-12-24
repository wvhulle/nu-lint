use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
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
    data: Option<String>,
    output_file: Option<String>,
}

impl HttpOptions {
    fn parse_curl<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        args.into_iter()
            .fold(
                (Self::default(), None::<&str>),
                |(mut opts, expecting), arg| match (expecting, arg) {
                    (Some("-X" | "--request"), method) => {
                        opts.method = parse_method(method);
                        (opts, None)
                    }
                    (Some("-H" | "--header"), header) => {
                        if let Some(h) = parse_header(header) {
                            opts.headers.push(h);
                        }
                        (opts, None)
                    }
                    (Some("-u" | "--user"), credentials) => {
                        let (user, password) = parse_credentials(credentials);
                        opts.user = user;
                        opts.password = password;
                        (opts, None)
                    }
                    (Some("-d" | "--data" | "--data-raw"), data) => {
                        opts.data = Some(data.to_string());
                        opts.method = match opts.method {
                            HttpMethod::Get => HttpMethod::Post,
                            m => m,
                        };
                        (opts, None)
                    }
                    (Some("-o" | "--output"), file) => {
                        opts.output_file = Some(file.to_string());
                        (opts, None)
                    }
                    (
                        None,
                        "-X" | "--request" | "-H" | "--header" | "-u" | "--user" | "-d" | "--data"
                        | "--data-raw" | "-o" | "--output",
                    ) => (opts, Some(arg)),
                    (None, s) if !s.starts_with('-') && opts.url.is_none() => {
                        opts.url = Some(s.to_string());
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
            parts.push(data.clone());
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

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let opts = HttpOptions::parse_curl(external_args_slices(args, context));
    let (replacement, description) = opts.to_nushell();

    Fix {
        explanation: description.into(),
        replacements: vec![Replacement {
            span: expr_span.into(),
            replacement_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "curl", NOTE, Some(build_fix))
}

pub const RULE: Rule = Rule::new(
    "use_builtin_curl",
    "Use Nushell's http commands instead of curl for better data handling",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/commands/docs/http_get.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
