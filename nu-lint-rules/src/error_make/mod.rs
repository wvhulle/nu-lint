use nu_lint_analysis::ast::external::literal_content;
use nu_protocol::ast::{Expr, Expression, RecordItem};

pub mod add_help_to_error;
pub mod add_label_to_error;
pub mod add_span_to_label;
pub mod add_url_to_error;
pub mod error_make_for_non_fatal;

#[must_use]
pub fn extract_field_name(key: &Expression) -> Option<&str> {
    literal_content(key)
}

#[must_use]
pub fn has_field(record: &[RecordItem], field_name: &str) -> bool {
    record.iter().any(|item| {
        matches!(item, RecordItem::Pair(key, _) if extract_field_name(key) == Some(field_name))
    })
}

#[must_use]
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

#[must_use]
pub fn get_labels_value(record: &[RecordItem]) -> Option<&Expression> {
    record.iter().find_map(|item| match item {
        RecordItem::Pair(key, value) => {
            matches!(extract_field_name(key), Some("labels" | "label")).then_some(value)
        }
        RecordItem::Spread(..) => None,
    })
}
