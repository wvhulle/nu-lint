use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'http get URL | save file' to download files. This provides structured \
                    data handling and better pipeline integration than wget.";

#[derive(Default)]
struct WgetOptions {
    url: Option<String>,
    output_file: Option<String>,
}

impl WgetOptions {
    fn parse_wget<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        args.into_iter()
            .fold(
                (Self::default(), None::<&str>),
                |(mut opts, expecting), arg| match (expecting, arg) {
                    (Some("-O" | "--output-document"), file) => {
                        opts.output_file = Some(file.to_string());
                        (opts, None)
                    }
                    (None, "-O" | "--output-document") => (opts, Some(arg)),
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
        let mut replacement = format!("http get {url}");
        if let Some(file) = &self.output_file {
            replacement = format!("{replacement} | save {file}");
        }
        let description = if self.output_file.is_some() {
            "Replace wget with 'http get | save'. Downloads return structured data that can be \
             processed before saving."
                .to_string()
        } else {
            "Replace wget with 'http get'. Use '| save <file>' to persist downloads. Nushell's \
             http returns structured data and integrates with pipelines."
                .to_string()
        };
        (replacement, description)
    }
}

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let opts = WgetOptions::parse_wget(external_args_slices(args, context));
    let (replacement, description) = opts.to_nushell();

    Fix {
        explanation: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            replacement_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_wget",
        "wget",
        NOTE,
        Some(build_fix),
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_wget",
        "Use 'http get | save' instead of wget",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn detects_wget_download() {
        let source = r"^wget https://example.com/file.tar.gz";
        rule().assert_replacement_contains(source, "http get");
        rule().assert_fix_explanation_contains(source, "save");
    }

    #[test]
    fn detects_wget_with_output() {
        let source = r"^wget -O output.html https://example.com";
        rule().assert_replacement_contains(source, "http get");
        rule().assert_replacement_contains(source, "| save output.html");
    }

    #[test]
    fn ignores_http_commands() {
        rule().assert_ignores(r"http get https://example.com/file.tar.gz | save file.tar.gz");
    }
}
