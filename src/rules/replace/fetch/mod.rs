use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'http get' for fetching URLs. It returns structured data and integrates \
                    with Nushell pipelines.";

#[derive(Default)]
struct FetchOptions {
    url: Option<String>,
}

impl FetchOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        if let Some(url) = args.into_iter().next()
            && !url.starts_with('-')
        {
            opts.url = Some(url.to_string());
        }
        opts
    }

    fn to_nushell(&self) -> (String, String) {
        let url = self.url.as_deref().unwrap_or("URL");
        (
            format!("http get {url}"),
            "Replace fetch with 'http get'. Nushell's http get returns structured data and \
             integrates with pipelines."
                .to_string(),
        )
    }
}

fn build_fix(
    _cmd_text: &str,
    _builtin_cmd: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let opts = FetchOptions::parse(external_args_slices(args, context));
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
        "prefer_builtin_fetch",
        "fetch",
        "http get",
        NOTE,
        Some(build_fix),
    )
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_fetch",
        "Use 'http get' instead of fetch",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn detects_fetch() {
        let source = r"^fetch https://api.example.com/data";
        rule().assert_replacement_contains(source, "http get");
    }

    #[test]
    fn ignores_http_get() {
        rule().assert_ignores(r"http get https://api.example.com");
    }
}
