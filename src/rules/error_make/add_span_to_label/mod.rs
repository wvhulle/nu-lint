use nu_protocol::ast::{Expr, Expression, ListItem, RecordItem};

use super::{extract_record_from_expr, get_labels_value, has_field};
use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

/// Check if a label record has 'text' but is missing 'span'
fn label_missing_span(record: &[RecordItem], context: &LintContext) -> bool {
    let has_text = has_field(record, "text", context);
    let has_span = has_field(record, "span", context);
    log::debug!("label_missing_span: has_text={has_text}, has_span={has_span}");
    has_text && !has_span
}

const fn list_item_expr(item: &ListItem) -> &Expression {
    match item {
        ListItem::Item(e) | ListItem::Spread(_, e) => e,
    }
}

/// Extract label records from labels expression (handles both single record and
/// list)
fn extract_labels_missing_span<'a>(
    labels_expr: &'a Expression,
    context: &LintContext,
) -> Vec<&'a Expression> {
    log::debug!("extract_labels_missing_span: expr = {:?}", labels_expr.expr);

    // Handle FullCellPath wrapping by extracting the head expression
    let effective_expr = match &labels_expr.expr {
        Expr::FullCellPath(full_cell_path) if full_cell_path.tail.is_empty() => {
            log::debug!("Unwrapping FullCellPath to get head expression");
            &full_cell_path.head
        }
        _ => labels_expr,
    };

    match &effective_expr.expr {
        // Single label record: labels: { text: "...", span: ... }
        Expr::Record(record) => {
            log::debug!("Found Record with {} items", record.len());
            let missing = label_missing_span(record, context);
            log::debug!("label_missing_span returned {missing}");
            if missing {
                vec![effective_expr]
            } else {
                vec![]
            }
        }
        // List of labels: labels: [{ text: "..." }, ...]
        Expr::List(items) => {
            log::debug!("Found List with {} items", items.len());
            items
                .iter()
                .filter(|item| {
                    extract_record_from_expr(list_item_expr(item))
                        .is_some_and(|record| label_missing_span(record, context))
                })
                .map(list_item_expr)
                .collect()
        }
        other => {
            log::debug!("Found other expr type: {other:?}");
            vec![]
        }
    }
}

struct AddSpanToLabel;

impl DetectFix for AddSpanToLabel {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "add_span_to_label"
    }

    fn short_description(&self) -> &'static str {
        "labels should include 'span' to highlight error location in user code"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/creating_errors.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        log::debug!("add_span_to_label detect() called");
        Self::no_fix(context.detect(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            if !call.is_call_to_command("error make", ctx) {
                return vec![];
            }
            log::debug!("Found error make call");

            let Some(error_record) = call
                .get_first_positional_arg()
                .and_then(extract_record_from_expr)
            else {
                log::debug!("No error record found");
                return vec![];
            };
            log::debug!("Found error record with {} items", error_record.len());

            let Some(labels_expr) = get_labels_value(error_record, ctx) else {
                log::debug!("No labels field found");
                return vec![];
            };
            log::debug!("Found labels field");

            let labels_missing_span = extract_labels_missing_span(labels_expr, ctx);
            log::debug!("labels_missing_span count = {}", labels_missing_span.len());
            if labels_missing_span.is_empty() {
                log::debug!("No labels missing span, returning empty");
                return vec![];
            }
            log::debug!("Found {} labels missing span", labels_missing_span.len());

            labels_missing_span
                .into_iter()
                .map(|label_expr| {
                    Detection::from_global_span(
                        "label has 'text' but is missing 'span' to highlight error location",
                        label_expr.span,
                    )
                    .with_primary_label("missing span")
                })
                .collect()
        }))
    }
}

pub static RULE: &dyn Rule = &AddSpanToLabel;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
