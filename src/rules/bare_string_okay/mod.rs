use std::collections::BTreeSet;

use nu_protocol::{Span, ast::{Block, Expr, Expression, ListItem, RecordItem}};

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

/// Recursively collect spans of expressions in command position.
/// Bare words in command position would be interpreted as commands, not strings.
fn collect_command_positions(block: &Block, ctx: &LintContext, spans: &mut BTreeSet<Span>) {
    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            if matches!(element.expr.expr, Expr::String(_)) {
                spans.insert(element.expr.span);
            }
            visit_expr(&element.expr, ctx, spans);
        }
    }
}

fn visit_expr(expr: &Expression, ctx: &LintContext, spans: &mut BTreeSet<Span>) {
    match &expr.expr {
        Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) | Expr::RowCondition(id) => {
            collect_command_positions(ctx.working_set.get_block(*id), ctx, spans);
        }
        Expr::MatchBlock(arms) => {
            for (_, arm_expr) in arms {
                if matches!(arm_expr.expr, Expr::String(_)) {
                    spans.insert(arm_expr.span);
                }
                visit_expr(arm_expr, ctx, spans);
            }
        }
        Expr::Keyword(kw) => visit_expr(&kw.expr, ctx, spans),
        Expr::Call(call) => {
            for arg in &call.arguments {
                if let Some(e) = arg.expr() {
                    visit_expr(e, ctx, spans);
                }
            }
        }
        Expr::List(items) => {
            for item in items {
                match item {
                    ListItem::Item(e) | ListItem::Spread(_, e) => visit_expr(e, ctx, spans),
                }
            }
        }
        Expr::Record(items) => {
            for item in items {
                match item {
                    RecordItem::Pair(k, v) => {
                        visit_expr(k, ctx, spans);
                        visit_expr(v, ctx, spans);
                    }
                    RecordItem::Spread(_, e) => visit_expr(e, ctx, spans),
                }
            }
        }
        Expr::BinaryOp(lhs, _, rhs) => {
            visit_expr(lhs, ctx, spans);
            visit_expr(rhs, ctx, spans);
        }
        Expr::FullCellPath(fcp) => visit_expr(&fcp.head, ctx, spans),
        _ => {}
    }
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
        return None;
    }

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
        "unnecessary_string_quotes"
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
        let mut command_positions = BTreeSet::new();
        collect_command_positions(context.ast, context, &mut command_positions);

        context.detect_with_fix_data(|expr, ctx| {
            if !matches!(expr.expr, Expr::String(_)) {
                return vec![];
            }

            // Skip strings in command position - they would be interpreted as commands
            if command_positions.contains(&expr.span) {
                return vec![];
            }

            check_string_needs_quotes(expr, ctx).into_iter().collect()
        })
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
