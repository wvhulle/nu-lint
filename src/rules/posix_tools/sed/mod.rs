use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

fn check(context: &LintContext) -> Vec<Violation> {
    let mut out = Vec::new();
    out.extend(detect_external_commands(
        context,
        "sed",
        "Use 'str replace' for text substitution",
        Some(build_fix),
    ));
    out.extend(detect_external_commands(
        context,
        "gsed",
        "Use 'str replace' for text substitution",
        Some(build_fix),
    ));
    out
}

pub const RULE: Rule = Rule::new(
    "use_builtin_sed",
    "Use Nu's 'str replace' instead of 'sed'",
    check,
    LintLevel::Warning,
)
.with_doc_url("https://www.nushell.sh/commands/docs/str_replace.html");

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let replacement = parse_sed_args(external_args_slices(args, context));

    Fix {
        explanation: "Replace with str replace".into(),
        replacements: vec![Replacement {
            span: expr_span.into(),
            replacement_text: replacement.into(),
        }],
    }
}

fn parse_sed_args<'a>(args: impl IntoIterator<Item = &'a str>) -> String {
    let mut pattern = None;
    let mut global = false;
    let mut file = None;
    let mut in_place = false;
    let mut regex_mode = false;
    let mut expect_expression = false;

    for arg in args {
        if expect_expression {
            pattern = Some(arg);
            global = arg.contains("/g");
            expect_expression = false;
            continue;
        }

        match arg {
            "-i" | "--in-place" => in_place = true,
            "-e" | "--expression" => expect_expression = true,
            "-E" | "-r" | "--regexp-extended" => regex_mode = true,
            "-n" | "--quiet" | "--silent" => {}
            s if s.starts_with('-') && !s.starts_with("--") => {
                (in_place, regex_mode, expect_expression) =
                    parse_combined_flags(s, in_place, regex_mode);
            }
            s if pattern.is_none() => {
                pattern = Some(s);
                global = s.contains("/g");
            }
            s => file = Some(s),
        }
    }

    let (find, replace) = parse_substitution(pattern.unwrap_or(""));
    let mut flags = String::new();
    if global {
        flags.push_str(" --all");
    }
    if regex_mode {
        flags.push_str(" --regex");
    }

    match (file, in_place) {
        (Some(f), true) => {
            format!("open {f} | str replace{flags} '{find}' '{replace}' | save -f {f}")
        }
        (Some(f), false) => format!("open {f} | str replace{flags} '{find}' '{replace}'"),
        _ => format!("str replace{flags} '{find}' '{replace}'"),
    }
}

fn parse_combined_flags(arg: &str, mut in_place: bool, mut regex_mode: bool) -> (bool, bool, bool) {
    let mut expect_expression = false;
    for ch in arg.chars().skip(1) {
        match ch {
            'i' => in_place = true,
            'e' => expect_expression = true,
            'E' | 'r' => regex_mode = true,
            _ => {}
        }
    }
    (in_place, regex_mode, expect_expression)
}

fn parse_substitution(pattern: &str) -> (&str, &str) {
    let clean = pattern.trim_matches('"').trim_matches('\'');

    if clean.starts_with('s') && clean.contains('/') {
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() >= 3 {
            return (parts[1], parts[2]);
        }
    }

    ("pattern", "replacement")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
