use std::collections::HashMap;

use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    ast::ext_command::{BuiltinAlternative, ExternalArgumentExt, detect_external_commands},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert(
        "curl",
        BuiltinAlternative::with_note(
            "http",
            "Use 'http get', 'http post', etc. for HTTP requests. Nushell's http commands return \
             structured data, integrate better with pipelines, and provide consistent \
             authentication and header handling.",
        ),
    );
    map.insert(
        "wget",
        BuiltinAlternative::with_note(
            "http get | save",
            "Use 'http get URL | save file' to download files. This provides structured data \
             handling and better pipeline integration than wget.",
        ),
    );
    map.insert(
        "fetch",
        BuiltinAlternative::with_note(
            "http get",
            "Use 'http get' for fetching URLs. It returns structured data and integrates \
             seamlessly with Nushell pipelines.",
        ),
    );
    map
}

#[derive(Default)]
struct HttpOptions {
    method: HttpMethod,
    url: Option<String>,
    headers: Vec<(String, String)>,
    user: Option<String>,
    password: Option<String>,
    data: Option<String>,
    output_file: Option<String>,
    quiet: bool,
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

impl HttpOptions {
    fn parse_curl(args: &[String]) -> Self {
        args.iter()
            .fold(
                (Self::default(), None::<&str>),
                |(mut opts, expecting), arg| match (expecting, arg.as_str()) {
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
                    ) => (opts, Some(arg.as_str())),
                    (None, s) if !s.starts_with('-') && opts.url.is_none() => {
                        opts.url = Some(s.to_string());
                        (opts, None)
                    }
                    _ => (opts, None),
                },
            )
            .0
    }

    fn parse_wget(args: &[String]) -> Self {
        args.iter()
            .fold(
                (Self::default(), None::<&str>),
                |(mut opts, expecting), arg| match (expecting, arg.as_str()) {
                    (Some("-O" | "--output-document"), file) => {
                        opts.output_file = Some(file.to_string());
                        (opts, None)
                    }
                    (Some("--user"), user) => {
                        opts.user = Some(user.to_string());
                        (opts, None)
                    }
                    (Some("--password"), pass) => {
                        opts.password = Some(pass.to_string());
                        (opts, None)
                    }
                    (None, "-O" | "--output-document" | "--user" | "--password") => {
                        (opts, Some(arg.as_str()))
                    }
                    (None, "-q" | "--quiet") => {
                        opts.quiet = true;
                        (opts, None)
                    }
                    (None, s) if !s.starts_with('-') && opts.url.is_none() => {
                        opts.url = Some(s.to_string());
                        (opts, None)
                    }
                    _ => (opts, None),
                },
            )
            .0
    }

    fn to_nushell(&self, cmd: &str) -> (String, String) {
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

        let mut replacement = parts.join(" ");

        if let Some(file) = &self.output_file {
            replacement = format!("{replacement} | save {file}");
        }

        let description = self.build_description(cmd);

        (replacement, description)
    }

    fn build_description(&self, cmd: &str) -> String {
        let mut parts = Vec::new();

        match cmd {
            "curl" => {
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
                    parts
                        .push("Data: -d data → positional argument or structured data".to_string());
                }
            }
            "wget" => {
                parts.push("Replace wget with 'http get | save'.".to_string());
                parts.push(
                    "Downloads return structured data that can be processed before saving."
                        .to_string(),
                );

                if self.output_file.is_some() {
                    parts.push("-O file → | save file at the end of the pipeline".to_string());
                }

                if self.quiet {
                    parts.push("Note: http commands are already quiet by default".to_string());
                }
            }
            "fetch" => {
                parts.push("Replace fetch with 'http get'.".to_string());
                parts.push(
                    "Nushell's http get returns structured data and integrates with pipelines."
                        .to_string(),
                );
            }
            _ => {}
        }

        parts.push(
            "Benefits: structured data output, better pipeline integration, consistent API across \
             all HTTP methods."
                .to_string(),
        );

        parts.join(" ")
    }
}

fn build_fix(
    cmd_text: &str,
    _alternative: &BuiltinAlternative,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text = args.extract_as_strings(context);

    let opts = match cmd_text {
        "curl" => HttpOptions::parse_curl(&args_text),
        "wget" => HttpOptions::parse_wget(&args_text),
        "fetch" => {
            let mut opts = HttpOptions::default();
            if let Some(url) = args_text.first() {
                opts.url = Some(url.clone());
            }
            opts
        }
        _ => HttpOptions::default(),
    };

    let (replacement, description) = opts.to_nushell(cmd_text);

    Fix {
        description: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            new_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_http",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_http",
        "Use Nushell's http commands instead of curl/wget/fetch for better data handling",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
