use nu_protocol::{
    Span,
    ast::{Expr, Expression},
};

use crate::{
    LintLevel,
    ast::string::{StringFormat, bare_word_needs_quotes},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    quoted_span: Span,
    unquoted_content: String,
}

/// Check if an expression is in command position.
/// A bare word in command position would be interpreted as a command to
/// execute.
fn is_in_command_position(expr: &Expression, parent: Option<&Expression>) -> bool {
    parent.is_none_or(|parent| match &parent.expr {
        // If parent is an ExternalCall and this is the head, it's in command position
        Expr::ExternalCall(head, _args) => head.span == expr.span,
        // If parent is a MatchBlock, this expression is a match arm body
        Expr::Block(_) | Expr::Closure(_) | Expr::Subexpression(_) | Expr::MatchBlock(_) => true,
        // Otherwise, not in command position
        _ => false,
    })
}

fn check_string_needs_quotes(expr: &Expression, ctx: &LintContext) -> Option<(Detection, FixData)> {
    let string_format = StringFormat::from_expression(expr, ctx)?;

    let (content, quote_type) = match &string_format {
        StringFormat::Double(s) => (s, "double"),
        StringFormat::Single(s) => (s, "single"),
        StringFormat::BareWord(_)
        | StringFormat::InterpolationDouble(_)
        | StringFormat::InterpolationSingle(_)
        | StringFormat::Backtick(_)
        | StringFormat::Raw(_) => return None,
    };

    if bare_word_needs_quotes(content) {
        log::debug!("String needs quotes: {content}");
        return None;
    }
    log::debug!("String does not need quotes: {content}");

    let violation = Detection::from_global_span(
        format!("Unnecessary {quote_type} quotes around string '{content}'"),
        expr.span,
    )
    .with_primary_label("can be a bare word")
    .with_help(
        "Bare words work for alphanumeric strings, paths starting with `.` or `/`, URLs, and \
         identifiers with `-` or `_`. Quotes are needed for strings with spaces, special \
         characters, or values that would be parsed as numbers, booleans, or null."
            .to_string(),
    );

    Some((
        violation,
        FixData {
            quoted_span: expr.span,
            unquoted_content: content.clone(),
        },
    ))
}

struct UnnecessaryStringQuotes;

impl DetectFix for UnnecessaryStringQuotes {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "remove_string_quotes"
    }

    fn explanation(&self) -> &'static str {
        "Quoted strings that can be written as bare words should omit the quotes for readability"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/working_with_strings.html#bare-word-strings")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut results = Vec::new();

        context.traverse_with_parent(|expr, parent| {
            if !matches!(expr.expr, Expr::String(_)) {
                return;
            }

            // Skip strings in command position - they would be interpreted as external
            // commands
            if is_in_command_position(expr, parent) {
                log::debug!(
                    "Skipping {} since it is in command position.",
                    context.get_span_text(expr.span)
                );
                return;
            }

            if let Some(detection) = check_string_needs_quotes(expr, context) {
                results.push(detection);
            }
        });

        results
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            format!("Remove quotes from '{}'", fix_data.unquoted_content),
            vec![Replacement::new(
                fix_data.quoted_span,
                fix_data.unquoted_content.clone(),
            )],
        ))
    }
}

pub static RULE: &dyn Rule = &UnnecessaryStringQuotes;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
