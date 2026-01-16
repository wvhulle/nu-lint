use nu_protocol::{
    Span,
    ast::{Expr, ListItem},
};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct CommaFixData {
    comma_span: Span,
}

fn find_comma_outside_comment(between_text: &str) -> Option<usize> {
    let mut offset = 0;
    for line in between_text.split('\n') {
        let comment_start = line.find('#').unwrap_or(line.len());
        if let Some(comma_pos) = line[..comment_start].find(',') {
            return Some(offset + comma_pos);
        }
        offset += line.len() + 1;
    }
    None
}

fn check_list_commas(
    context: &LintContext,
    span: Span,
    items: &[ListItem],
) -> Vec<(Detection, CommaFixData)> {
    let mut violations = Vec::new();
    if items.len() < 2 {
        return violations;
    }
    let list_text = context.span_text(span);
    if !list_text.trim_start().starts_with('[') {
        return violations;
    }
    for i in 0..items.len() - 1 {
        let current_expr = match &items[i] {
            ListItem::Item(expr) | ListItem::Spread(_, expr) => expr,
        };
        let next_expr = match &items[i + 1] {
            ListItem::Item(expr) | ListItem::Spread(_, expr) => expr,
        };
        if current_expr.span.end >= next_expr.span.start {
            continue;
        }
        let between_span = Span::new(current_expr.span.end, next_expr.span.start);
        let between_text = context.span_text(between_span);
        if let Some(comma_pos) = find_comma_outside_comment(between_text) {
            let comma_span = Span::new(
                between_span.start + comma_pos,
                between_span.start + comma_pos + 1,
            );
            violations.push((
                Detection::from_global_span("Omit commas between list items", comma_span),
                CommaFixData { comma_span },
            ));
        }
    }
    violations
}

struct OmitListCommas;

impl DetectFix for OmitListCommas {
    type FixInput<'a> = CommaFixData;

    fn id(&self) -> &'static str {
        "omit_list_commas"
    }

    fn short_description(&self) -> &'static str {
        "Omit commas between list items."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#basic")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::List(items) => check_list_commas(ctx, expr.span, items),
            _ => vec![],
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            "Remove comma",
            vec![Replacement::new(fix_data.comma_span, String::new())],
        ))
    }
}

pub static RULE: &dyn Rule = &OmitListCommas;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
