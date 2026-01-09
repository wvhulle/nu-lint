use nu_protocol::ast::{Expr, Expression, RecordItem};

use crate::{
    ast::{expression::ExpressionExt, string::StringFormat},
    context::LintContext,
};

pub mod add_help;
pub mod add_label;
pub mod add_span_to_label;
pub mod add_url;
pub mod non_fatal_catch;

pub fn extract_field_name(key: &Expression, context: &LintContext) -> String {
    StringFormat::from_expression(key, context)
        .map(|format| format.content().to_string())
        .unwrap_or_else(|| key.span_text(context).to_string())
}

pub fn has_field(record: &[RecordItem], field_name: &str, context: &LintContext) -> bool {
    record.iter().any(|item| {
        matches!(item, RecordItem::Pair(key, _) if extract_field_name(key, context) == field_name)
    })
}

pub fn extract_record_from_expr(expr: &Expression) -> Option<&Vec<RecordItem>> {
    match &expr.expr {
        Expr::Record(record) => Some(record),
        Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
            Expr::Record(record) => Some(record),
            _ => None,
        },
        _ => None,
    }
}

pub fn get_labels_value<'a>(
    record: &'a [RecordItem],
    context: &LintContext,
) -> Option<&'a Expression> {
    record.iter().find_map(|item| match item {
        RecordItem::Pair(key, value) => {
            let name = extract_field_name(key, context);
            (name == "labels" || name == "label").then_some(value)
        }
        RecordItem::Spread(..) => None,
    })
}
