use nu_protocol::ast::{Expr, Expression, Traverse};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct EscapeMatch {
    span: nu_protocol::Span,
    ansi_name: &'static str,
}

struct AnsiEscapeSequence {
    string_span: nu_protocol::Span,
    escapes: Vec<EscapeMatch>,
}

/// ANSI escape sequence patterns as they appear in source code (with literal
/// \e).
const SOURCE_ESCAPE_PATTERNS: &[(&str, &str)] = &[
    // Standard colors (30-37)
    ("\\e[30m", "black"),
    ("\\e[31m", "red"),
    ("\\e[32m", "green"),
    ("\\e[33m", "yellow"),
    ("\\e[34m", "blue"),
    ("\\e[35m", "magenta"),
    ("\\e[36m", "cyan"),
    ("\\e[37m", "white"),
    // Bright/light colors (90-97)
    ("\\e[90m", "dark_gray"),
    ("\\e[91m", "red_bold"),
    ("\\e[92m", "green_bold"),
    ("\\e[93m", "yellow_bold"),
    ("\\e[94m", "blue_bold"),
    ("\\e[95m", "magenta_bold"),
    ("\\e[96m", "cyan_bold"),
    ("\\e[97m", "white_bold"),
    // Style attributes
    ("\\e[0m", "reset"),
    ("\\e[1m", "bold"),
    ("\\e[2m", "dimmed"),
    ("\\e[3m", "italic"),
    ("\\e[4m", "underline"),
    ("\\e[5m", "blink"),
    ("\\e[7m", "reverse"),
    ("\\e[8m", "hidden"),
    ("\\e[9m", "strikethrough"),
];

/// ANSI escape sequence patterns as interpreted (with actual ESC byte).
const INTERPRETED_ESCAPE_PATTERNS: &[(&str, &str)] = &[
    ("\x1b[30m", "black"),
    ("\x1b[31m", "red"),
    ("\x1b[32m", "green"),
    ("\x1b[33m", "yellow"),
    ("\x1b[34m", "blue"),
    ("\x1b[35m", "magenta"),
    ("\x1b[36m", "cyan"),
    ("\x1b[37m", "white"),
    ("\x1b[90m", "dark_gray"),
    ("\x1b[91m", "red_bold"),
    ("\x1b[92m", "green_bold"),
    ("\x1b[93m", "yellow_bold"),
    ("\x1b[94m", "blue_bold"),
    ("\x1b[95m", "magenta_bold"),
    ("\x1b[96m", "cyan_bold"),
    ("\x1b[97m", "white_bold"),
    ("\x1b[0m", "reset"),
    ("\x1b[1m", "bold"),
    ("\x1b[2m", "dimmed"),
    ("\x1b[3m", "italic"),
    ("\x1b[4m", "underline"),
    ("\x1b[5m", "blink"),
    ("\x1b[7m", "reverse"),
    ("\x1b[8m", "hidden"),
    ("\x1b[9m", "strikethrough"),
];

fn has_ansi_escapes(text: &str) -> bool {
    INTERPRETED_ESCAPE_PATTERNS
        .iter()
        .any(|(pattern, _)| text.contains(pattern))
}

fn find_source_escapes(source: &str, base_offset: usize) -> Vec<EscapeMatch> {
    let mut results = Vec::new();

    for (pattern, ansi_name) in SOURCE_ESCAPE_PATTERNS {
        let mut start = 0;
        while let Some(pos) = source[start..].find(pattern) {
            let absolute_pos = start + pos;
            let span = nu_protocol::Span::new(
                base_offset + absolute_pos,
                base_offset + absolute_pos + pattern.len(),
            );
            results.push(EscapeMatch { span, ansi_name });
            start = absolute_pos + pattern.len();
        }
    }

    results.sort_by_key(|m| m.span.start);
    results
}

fn check_plain_string(
    text: &str,
    span: nu_protocol::Span,
    ctx: &LintContext,
) -> Option<(Detection, AnsiEscapeSequence)> {
    if !has_ansi_escapes(text) {
        return None;
    }

    let source_text = ctx.span_text(span);

    let escapes = find_source_escapes(source_text, span.start);

    if escapes.is_empty() {
        return None;
    }

    let violation = Detection::from_global_span(
        "Use `ansi` command instead of raw ANSI escape sequences",
        span,
    )
    .with_primary_label(if escapes.len() == 1 {
        "ANSI escape sequence"
    } else {
        "ANSI escape sequences"
    });

    Some((
        violation,
        AnsiEscapeSequence {
            string_span: span,
            escapes,
        },
    ))
}

fn check_expression(
    expr: &Expression,
    context: &LintContext,
) -> Vec<(Detection, AnsiEscapeSequence)> {
    let mut violations = Vec::new();

    match &expr.expr {
        Expr::String(text) => {
            if let Some(violation) = check_plain_string(text, expr.span, context) {
                violations.push(violation);
            }
        }
        Expr::StringInterpolation(parts) => {
            let string_parts = parts.iter().filter_map(|part| match &part.expr {
                Expr::String(text) => Some((text.as_str(), part.span)),
                _ => None,
            });
            for (text, span) in string_parts {
                if let Some(violation) = check_plain_string(text, span, context) {
                    violations.push(violation);
                }
            }
        }
        _ => {}
    }

    violations
}

struct AnsiOverEscapeCodes;

impl DetectFix for AnsiOverEscapeCodes {
    type FixInput<'a> = AnsiEscapeSequence;

    fn id(&self) -> &'static str {
        "ansi_over_escape_codes"
    }

    fn short_description(&self) -> &'static str {
        "Raw ANSI escape replaceable with `ansi`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/ansi.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| check_expression(expr, context),
            &mut violations,
        );

        violations
    }

    fn fix(&self, _ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let escapes = &fix_data.escapes;

        if escapes.len() != 2 {
            return None;
        }

        let first = &escapes[0];
        let second = &escapes[1];

        // Must be color+reset pattern
        if second.ansi_name != "reset" && first.ansi_name != "reset" {
            return None;
        }

        let color_name = if first.ansi_name == "reset" {
            second.ansi_name
        } else {
            first.ansi_name
        };

        // Build replacements: replace each escape sequence span with (ansi name)
        // and change the opening quote from " to $"
        let mut replacements = Vec::with_capacity(3);

        // Replace opening " with $"
        let opening_quote_span =
            nu_protocol::Span::new(fix_data.string_span.start, fix_data.string_span.start + 1);
        replacements.push(Replacement::new(opening_quote_span, "$\"".to_string()));

        // Replace each escape sequence with (ansi name)
        for escape in escapes {
            let replacement = format!("(ansi {})", escape.ansi_name);
            replacements.push(Replacement::new(escape.span, replacement));
        }

        Some(Fix {
            explanation: format!(
                "Replace ANSI escape sequences with `ansi {color_name}` and `ansi reset` commands"
            )
            .into(),
            replacements,
        })
    }
}

pub static RULE: &dyn Rule = &AnsiOverEscapeCodes;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
