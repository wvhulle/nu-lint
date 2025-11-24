use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str =
    "Use 'where' for filtering, 'select' for columns, or 'each' for row-by-row processing.";

#[derive(Default)]
struct AwkOptions {
    fs: Option<String>,
    pattern: Option<String>,
    print_field: Option<usize>,
    files: Vec<String>,
}

fn strip_quotes(s: &str) -> &str {
    let t = s.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        &t[1..t.len() - 1]
    } else {
        t
    }
}

fn parse_program(program: &str) -> (Option<String>, Option<usize>) {
    let p = program.trim();
    // handle formats: /regex/ {print $N} | {print $N} | /regex/
    let mut pat: Option<String> = None;
    let mut print: Option<usize> = None;

    // extract /regex/
    if let Some((start, end)) = p
        .find('/')
        .and_then(|s| p[s + 1..].find('/').map(|er| (s, s + 1 + er)))
    {
        pat = Some(p[start + 1..end].to_string());
    }

    // extract print $N inside braces or alone
    let body = p
        .trim_start_matches(|c: char| c == '{' || c.is_whitespace())
        .trim_end_matches(|c: char| c == '}' || c.is_whitespace());
    if let Some(idx) = body.find("print $") {
        let rest = &body[idx + 7..];
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(n) = digits.parse::<usize>() {
            print = Some(n);
        }
    }

    (pat, print)
}

fn parse_awk<'a>(args: impl IntoIterator<Item = &'a str>) -> AwkOptions {
    let mut opts = AwkOptions::default();
    let mut iter = args.into_iter().peekable();
    while let Some(arg) = iter.next() {
        match arg {
            "-F" => {
                if let Some(sep) = iter.next() {
                    opts.fs = Some(strip_quotes(sep).to_string());
                }
            }
            s if s.starts_with("-F") && s.len() > 2 => {
                opts.fs = Some(strip_quotes(&s[2..]).to_string());
            }
            s if s.starts_with('"') || s.starts_with('\'') => {
                let prog = strip_quotes(s);
                let (pat, print) = parse_program(prog);
                if pat.is_some() {
                    opts.pattern = pat;
                }
                if print.is_some() {
                    opts.print_field = print;
                }
            }
            s if !s.starts_with('-') => opts.files.push(s.to_string()),
            _ => {}
        }
    }
    opts
}

fn build_replacement(opts: &AwkOptions) -> (String, String) {
    // Base
    let mut parts: Vec<String> = Vec::new();
    if let Some(file) = opts.files.first() {
        parts.push(format!("open {file} | lines"));
    } else {
        parts.push("lines".to_string());
    }

    // Pattern filter
    if let Some(pat) = &opts.pattern {
        parts.push(format!("where $it =~ \"{pat}\""));
    }

    // Print field
    if let Some(n) = opts.print_field {
        let col = format!("column{n}");
        let sep = opts.fs.as_deref().unwrap_or(" ");
        if sep == " " {
            parts.push("split column \" \"".to_string());
        } else {
            parts.push(format!("split column {sep}"));
        }
        parts.push(format!("get {col}"));
    } else {
        // generic map
        parts.push("each {|it| $it }".to_string());
    }

    let replacement = parts.join(" | ");
    let explanation = "Convert awk to Nushell pipeline".to_string();
    (replacement, explanation)
}

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let opts = parse_awk(external_args_slices(args, context));
    let (replacement, description) = build_replacement(&opts);
    Fix::with_explanation(description, vec![Replacement::new(expr_span, replacement)])
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "prefer_builtin_awk", "awk", NOTE, Some(build_fix))
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_awk",
        "Use Nushell pipelines (where/select/each) instead of awk",
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn converts_awk_to_nu_pipeline() {
        let source = "^awk";
        rule().assert_replacement_contains(source, "lines | each");
        rule().assert_fix_explanation_contains(source, "pipeline");
    }

    #[test]
    fn converts_awk_print_first_field() {
        let source = "^awk '{print $1}' input.txt";
        rule().assert_replacement_contains(
            source,
            "open input.txt | lines | split column \" \" | get column1",
        );
    }

    #[test]
    fn converts_awk_with_field_separator() {
        let source = "^awk -F, '{print $2}' data.csv";
        rule().assert_replacement_contains(
            source,
            "open data.csv | lines | split column , | get column2",
        );
    }

    #[test]
    fn converts_awk_filter_pattern() {
        let source = "^awk '/error/' logfile";
        rule().assert_replacement_contains(source, "open logfile | lines | where $it =~ \"error\"");
    }
}
