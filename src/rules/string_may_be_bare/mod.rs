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

/// Check if an expression is in command position where a bare word would be
/// interpreted as a command to execute.
///
/// Per Nushell docs: "if you use a bare word plainly on the command line (that
/// is, not inside a data structure or used as a command parameter) or inside
/// round brackets ( ), it will be interpreted as an external command"
fn is_in_command_position(expr: &Expression, parent: Option<&Expression>) -> bool {
    parent.is_none_or(|parent| match &parent.expr {
        // If parent is an ExternalCall and this is the head, it's in command position
        Expr::ExternalCall(head, _args) => head.span == expr.span,
        // If parent is a MatchBlock, this expression is a match arm body (command position)
        Expr::Block(_) | Expr::Closure(_) | Expr::Subexpression(_) | Expr::MatchBlock(_) => true,
        // Otherwise, not in command position
        _ => false,
    })
}

struct UnnecessaryStringQuotes;

impl DetectFix for UnnecessaryStringQuotes {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "string_may_be_bare"
    }

    fn short_description(&self) -> &'static str {
        "Quoted string can be bare word"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/working_with_strings.html#bare-word-strings")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut results = Vec::new();

        context.traverse_with_parent(|expr, parent| {
            // Only look at string expressions
            let Expr::String(_) = &expr.expr else {
                return;
            };

            // Skip strings in command position - they would be interpreted as external
            // commands
            if is_in_command_position(expr, parent) {
                log::debug!(
                    "Skipping {} - in command position",
                    context.span_text(expr.span)
                );
                return;
            }

            // Extract the string content and quote type
            let Some(string_format) = StringFormat::from_expression(expr, context) else {
                return;
            };

            let (unquoted_content, quote_type) = match &string_format {
                StringFormat::Double(s) => (s, "double"),
                StringFormat::Single(s) => (s, "single"),
                // Only suggest removing simple quotes, not interpolation/raw/backtick strings
                StringFormat::BareWord(_)
                | StringFormat::InterpolationDouble(_)
                | StringFormat::InterpolationSingle(_)
                | StringFormat::Backtick(_)
                | StringFormat::Raw(_) => return,
            };

            // Check if the string actually needs quotes
            if bare_word_needs_quotes(unquoted_content) {
                log::debug!("String '{unquoted_content}' needs quotes");
                return;
            }

            log::debug!("String '{unquoted_content}' can be a bare word");

            // Report unnecessary quotes
            let violation = Detection::from_global_span(
                format!("Unnecessary {quote_type} quotes around '{unquoted_content}'"),
                expr.span,
            )
            .with_primary_label("can be a bare word");

            results.push((
                violation,
                FixData {
                    quoted_span: expr.span,
                    unquoted_content: unquoted_content.clone(),
                },
            ));
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
