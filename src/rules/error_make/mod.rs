use nu_protocol::ast::{Expr, Expression, RecordItem};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
};

pub mod add_help;
pub mod add_label;
pub mod add_span_to_label;
pub mod add_url;
pub mod non_fatal_catch;

pub fn extract_field_name(key: &Expression, context: &LintContext) -> String {
    match &key.expr {
        Expr::String(s) | Expr::RawString(s) => s.clone(),
        _ => key
            .span_text(context)
            .trim_matches(|c| c == '"' || c == '\'')
            .to_string(),
    }
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

pub fn extract_first_function_parameter(
    context: &LintContext,
    call_span: nu_protocol::Span,
) -> Option<String> {
    context
        .ast
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .filter_map(|element| match &element.expr.expr {
            Expr::Call(call) => Some(call),
            _ => None,
        })
        .find_map(|call| {
            call.custom_command_def(context)?;
            let block_id = call.get_positional_arg(2)?.extract_block_id()?;
            let func_block = context.working_set.get_block(block_id);

            func_block
                .span
                .filter(|s| s.contains_span(call_span))
                .and_then(|_| {
                    func_block
                        .signature
                        .required_positional
                        .first()
                        .and_then(|param| param.var_id)
                        .map(|var_id| {
                            let var_span =
                                context.working_set.get_variable(var_id).declaration_span;
                            context.get_span_text(var_span).to_string()
                        })
                })
        })
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
