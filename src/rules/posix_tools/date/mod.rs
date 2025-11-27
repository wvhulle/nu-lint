use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'date now' for current time, '| date to-timezone <TZ>' for timezone \
                    conversion, and '| format date \"%Y-%m-%d\"' to format. Use '... | into \
                    datetime' to parse strings.";

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text: Vec<&str> = external_args_slices(args, context).collect();

    // Parse external `date` flags
    let mut fmt: Option<String> = None; // +%Y-%m-%d
    let mut utc = false; // -u / --utc
    let mut date_str: Option<String> = None; // -d "..." / --date=...

    let mut iter = args_text.iter().copied().peekable();
    while let Some(arg) = iter.next() {
        if arg == "-u" || arg == "--utc" {
            utc = true;
        } else if let Some(rest) = arg.strip_prefix("--date=") {
            date_str = Some(rest.to_string());
        } else if arg == "-d" || arg == "--date" {
            if let Some(val) = iter.next() {
                date_str = Some(val.to_string());
            }
        } else if let Some(rest) = arg.strip_prefix('+') {
            // Keep original format; ensure it's quoted in Nu replacement
            let needs_quotes = !(rest.starts_with('"') || rest.starts_with('\''));
            let f = if needs_quotes {
                format!("'{rest}'")
            } else {
                rest.to_string()
            };
            fmt = Some(f);
        }
    }

    // Build Nushell pipeline
    let mut parts: Vec<String> = Vec::new();

    let had_date_str = date_str.is_some();
    if let Some(ds) = date_str {
        // Ensure quoted string literal for into datetime
        let trimmed = ds.trim();
        let ds_quoted = if trimmed.starts_with('\'') || trimmed.starts_with('"') {
            trimmed.to_string()
        } else {
            format!("'{trimmed}'")
        };
        parts.push(format!("{ds_quoted} | into datetime"));
    } else {
        parts.push("date now".to_string());
    }

    if utc {
        parts.push("date to-timezone UTC".to_string());
    }
    let had_fmt = fmt.is_some();
    if let Some(f) = fmt {
        parts.push(format!("format date {f}"));
    }

    let replacement = parts.join(" | ");
    let mut explanation = Vec::new();
    explanation.push("Replace external 'date' with Nushell date pipeline".to_string());
    explanation.push("returns a datetime value".to_string());
    if had_date_str {
        explanation.push("parse string with 'into datetime'".to_string());
    }
    if utc {
        explanation.push("convert to UTC with 'date to-timezone'".to_string());
    }
    if had_fmt {
        explanation.push("format output with 'format date'".to_string());
    }
    if explanation.len() == 1 {
        explanation.push("use 'date now' for current date/time".to_string());
    }

    Fix::with_explanation(
        explanation.join("; "),
        vec![Replacement::new(expr_span, replacement)],
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "date", NOTE, Some(build_fix))
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_date",
        "Use 'date now' instead of external date",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/date_now.html")
}

#[cfg(test)]
mod tests {
    use super::rule;

    #[test]
    fn converts_date_command_to_date_now() {
        let source = "^date";
        rule().assert_replacement_contains(source, "date now");
        rule().assert_fix_explanation_contains(source, "datetime");
    }

    #[test]
    fn converts_date_with_format_string() {
        // External date formatting like +%Y-%m-%d should still prefer 'date now'
        let source = "^date +%Y-%m-%d";
        rule().assert_replacement_contains(source, "date now");
        rule().assert_fix_explanation_contains(source, "datetime");
    }

    #[test]
    fn converts_date_with_utc_flag() {
        // UTC flag doesn't change the recommendation to use 'date now'
        let source = "^date -u";
        rule().assert_replacement_contains(source, "date now");
        rule().assert_fix_explanation_contains(source, "datetime");
    }

    #[test]
    fn ignores_builtin_date_now() {
        let source = "date now";
        rule().assert_ignores(source);
    }

    #[test]
    fn ignores_builtin_date_pipeline() {
        let source = "date now | format date '%Y-%m-%d'";
        rule().assert_ignores(source);
    }
}
