use std::fmt::Write;

use nu_protocol::ast::{Expr, Expression, Traverse};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct AnsiEscapeSequence {
    string_span: nu_protocol::Span,
    // Store all escapes: (position, length, ansi_name)
    all_escapes: Vec<(usize, usize, String)>,
}

/// ANSI escape sequence patterns with their corresponding `ansi` command names.
/// These mappings align with nu-ansi-term's Color enum and Nushell's ansi
/// command.
const fn get_ansi_escape_patterns() -> &'static [(&'static str, &'static str)] {
    &[
        // Standard colors (30-37) - match nu-ansi-term::Color variants
        ("\x1b[30m", "black"),   // Color::Black foreground
        ("\x1b[31m", "red"),     // Color::Red foreground
        ("\x1b[32m", "green"),   // Color::Green foreground
        ("\x1b[33m", "yellow"),  // Color::Yellow foreground
        ("\x1b[34m", "blue"),    // Color::Blue foreground
        ("\x1b[35m", "magenta"), // Color::Purple/Magenta foreground
        ("\x1b[36m", "cyan"),    // Color::Cyan foreground
        ("\x1b[37m", "white"),   // Color::White foreground
        // Bright/light colors (90-97) - match nu-ansi-term light variants
        ("\x1b[90m", "dark_gray"),    // Color::DarkGray foreground
        ("\x1b[91m", "red_bold"),     // Color::LightRed foreground
        ("\x1b[92m", "green_bold"),   // Color::LightGreen foreground
        ("\x1b[93m", "yellow_bold"),  // Color::LightYellow foreground
        ("\x1b[94m", "blue_bold"),    // Color::LightBlue foreground
        ("\x1b[95m", "magenta_bold"), // Color::LightPurple foreground
        ("\x1b[96m", "cyan_bold"),    // Color::LightCyan foreground
        ("\x1b[97m", "white_bold"),   // Color::LightGray foreground
        // Style attributes (SGR parameters)
        ("\x1b[0m", "reset"),         // Reset all attributes
        ("\x1b[1m", "bold"),          // Bold/increased intensity
        ("\x1b[2m", "dimmed"),        // Dimmed/faint
        ("\x1b[3m", "italic"),        // Italic
        ("\x1b[4m", "underline"),     // Underline
        ("\x1b[5m", "blink"),         // Slow blink
        ("\x1b[7m", "reverse"),       // Reverse video
        ("\x1b[8m", "hidden"),        // Hidden/invisible
        ("\x1b[9m", "strikethrough"), // Crossed out/strike-through
    ]
}

fn find_all_ansi_escapes(
    text: &str,
    _base_span: nu_protocol::Span,
) -> Vec<(usize, String, String)> {
    let mut results = Vec::new();
    let patterns = get_ansi_escape_patterns();

    for (escape_seq, ansi_name) in patterns {
        let mut start = 0;
        while let Some(pos) = text[start..].find(escape_seq) {
            let absolute_pos = start + pos;
            results.push((
                absolute_pos,
                (*escape_seq).to_string(),
                (*ansi_name).to_string(),
            ));
            start = absolute_pos + escape_seq.len();
        }
    }

    // Sort by position so violations appear in the order they occur in the string
    results.sort_by_key(|(pos, _, _)| *pos);

    results
}

fn find_source_escapes(source: &str) -> Vec<(usize, usize, String)> {
    let mut results = Vec::new();

    // Patterns as they appear in SOURCE (with literal \e)
    let patterns = [
        ("\\e[30m", "black"),
        ("\\e[31m", "red"),
        ("\\e[32m", "green"),
        ("\\e[33m", "yellow"),
        ("\\e[34m", "blue"),
        ("\\e[35m", "magenta"),
        ("\\e[36m", "cyan"),
        ("\\e[37m", "white"),
        ("\\e[90m", "dark_gray"),
        ("\\e[91m", "red_bold"),
        ("\\e[92m", "green_bold"),
        ("\\e[93m", "yellow_bold"),
        ("\\e[94m", "blue_bold"),
        ("\\e[95m", "magenta_bold"),
        ("\\e[96m", "cyan_bold"),
        ("\\e[97m", "white_bold"),
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

    for (pattern, ansi_name) in &patterns {
        let mut start = 0;
        while let Some(pos) = source[start..].find(pattern) {
            let absolute_pos = start + pos;
            results.push((absolute_pos, pattern.len(), (*ansi_name).to_string()));
            start = absolute_pos + pattern.len();
        }
    }

    results.sort_by_key(|(pos, _, _)| *pos);
    results
}

fn check_string_expression(
    expr: &Expression,
    context: &LintContext,
) -> Vec<(Detection, AnsiEscapeSequence)> {
    let mut violations = Vec::new();

    if let Expr::String(content) = &expr.expr {
        // Check for ANSI escapes in the interpreted content
        let escapes = find_all_ansi_escapes(content, expr.span);

        if escapes.is_empty() {
            return violations;
        }

        let has_reset = escapes.iter().any(|(_, _, name)| name == "reset");
        let total_escapes = escapes.len();

        // Get the first non-reset escape for the help message
        let example_ansi = escapes
            .iter()
            .find(|(_, _, name)| name != "reset")
            .map_or("red", |(_, _, name)| name.as_str());

        let reset_reminder = if has_reset {
            ""
        } else {
            "\n\nNote: Don't forget to add `$(ansi reset)` at the end to reset colors/styles."
        };

        let violation = Detection::from_global_span(
            "Use `ansi` command instead of raw ANSI escape sequences",
            expr.span,
        )
        .with_primary_label(if total_escapes == 1 {
            "ANSI escape sequence"
        } else {
            "ANSI escape sequences"
        })
        .with_help(format!(
            "Use the `ansi` command for clearer and more portable color output.\n\
             Instead of raw escape sequences, use `ansi {example_ansi}` or interpolate with `$(ansi {example_ansi})`.\n\
             \n\
             The `ansi` command provides:\n\
             - Named colors (red, green, blue, etc.)\n\
             - Styles (bold, underline, italic, etc.)\n\
             - Better readability and maintainability\n\
             - Cross-platform compatibility\n\
             \n\
             Example: `print $\"(ansi red)Error:(ansi reset) Something failed\"`{reset_reminder}\n\
             \n\
             See: https://www.nushell.sh/commands/docs/ansi.html"
        ));

        // For fixes, we need positions in the SOURCE text (with \e), not interpreted
        // text (with ESC byte) Get the source text and search for escape
        // patterns there
        let source_text = context.get_span_text(expr.span);
        let source_content = if source_text.starts_with('"') && source_text.ends_with('"') {
            &source_text[1..source_text.len() - 1]
        } else {
            source_text
        };

        // Search for \e[XXm patterns in the source
        let all_escapes = find_source_escapes(source_content);

        violations.push((
            violation,
            AnsiEscapeSequence {
                string_span: expr.span,
                all_escapes,
            },
        ));
    }

    violations
}

fn check_expression(
    expr: &Expression,
    context: &LintContext,
) -> Vec<(Detection, AnsiEscapeSequence)> {
    check_string_expression(expr, context)
}

struct AnsiOverEscapeCodes;

impl DetectFix for AnsiOverEscapeCodes {
    type FixInput<'a> = AnsiEscapeSequence;

    fn id(&self) -> &'static str {
        "ansi_over_escape_codes"
    }

    fn explanation(&self) -> &'static str {
        "Use `ansi` command instead of raw ANSI escape sequences for colored output"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/ansi.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
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

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let string_text = ctx.get_span_text(fix_data.string_span);
        let escapes = &fix_data.all_escapes;

        match escapes.len() {
            1 => {
                // Single escape sequence - simple replacement
                let (_, _, ansi_name) = &escapes[0];
                let fixed_text = format!("$(ansi {ansi_name})");

                Some(Fix::with_explanation(
                    format!(
                        "Replace string with ANSI escape sequence with `ansi {ansi_name}` command"
                    ),
                    vec![Replacement::new(fix_data.string_span, fixed_text)],
                ))
            }
            2 => {
                // Check if this is a color+reset pattern
                let (_pos1, _len1, name1) = &escapes[0];
                let (_pos2, _len2, name2) = &escapes[1];

                // Common pattern: color/style at start, reset at end
                (name2 == "reset" || name1 == "reset").then(|| {
                    let content = if string_text.starts_with('"') && string_text.ends_with('"') {
                        &string_text[1..string_text.len() - 1]
                    } else {
                        string_text
                    };

                    // Get the color/style name (the non-reset one)
                    let color_name = if name1 == "reset" { name2 } else { name1 };

                    // Reconstruct with $(ansi ...) interpolations
                    let mut result = String::new();
                    let mut last_end = 0;

                    for (pos, len, ansi_name) in escapes {
                        // Add text before this escape
                        result.push_str(&content[last_end..*pos]);
                        // Add $(ansi ...) interpolation
                        write!(&mut result, "$(ansi {ansi_name})").unwrap();
                        last_end = pos + len;
                    }

                    // Add any remaining text after the last escape
                    result.push_str(&content[last_end..]);

                    // Wrap in double quotes for interpolated string
                    let fixed_text = format!("\"{result}\"");
                    Fix::with_explanation(
                        format!(
                            "Replace string with ANSI escape sequences with `ansi {color_name}` \
                             and `ansi reset` commands"
                        ),
                        vec![Replacement::new(fix_data.string_span, fixed_text)],
                    )
                })
            }
            _ => {
                // Three or more escape sequences - manual rewriting is better
                None
            }
        }
    }
}

pub static RULE: &dyn Rule = &AnsiOverEscapeCodes;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
