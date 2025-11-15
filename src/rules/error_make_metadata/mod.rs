use nu_protocol::ast::{Call, Expr, Expression, RecordItem};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt, span::SpanExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

struct MetadataFields {
    has_msg: bool,
    has_label: bool,
    has_help: bool,
}

impl MetadataFields {
    fn missing_fields(&self) -> Vec<&'static str> {
        [
            (!self.has_label).then_some("label"),
            (!self.has_help).then_some("help"),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    const fn is_valid(&self) -> bool {
        self.has_msg && self.has_label && self.has_help
    }
}

fn extract_field_name(key: &Expression, context: &LintContext) -> String {
    match &key.expr {
        Expr::String(s) | Expr::RawString(s) => s.clone(),
        _ => key
            .span_text(context)
            .trim_matches(|c| c == '"' || c == '\'')
            .to_string(),
    }
}

fn extract_msg_value(record: &[RecordItem], context: &LintContext) -> Option<String> {
    record.iter().find_map(|item| match item {
        RecordItem::Pair(key, value) if extract_field_name(key, context) == "msg" => {
            Some(match &value.expr {
                Expr::String(s) | Expr::RawString(s) => s.clone(),
                _ => value.span_text(context).to_string(),
            })
        }
        _ => None,
    })
}

fn extract_first_function_parameter(
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
            let (_, _) = call.extract_function_definition(context)?;
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
                            context
                                .working_set
                                .get_variable(var_id)
                                .declaration_span
                                .text(context)
                                .to_string()
                        })
                })
        })
}

fn analyze_record_fields(record: &[RecordItem], context: &LintContext) -> MetadataFields {
    record
        .iter()
        .filter_map(|item| match item {
            RecordItem::Pair(key, _) => Some(extract_field_name(key, context)),
            RecordItem::Spread(..) => None,
        })
        .fold(
            MetadataFields {
                has_msg: false,
                has_label: false,
                has_help: false,
            },
            |mut fields, field_name| {
                match field_name.as_str() {
                    "msg" => fields.has_msg = true,
                    "label" => fields.has_label = true,
                    "help" => fields.has_help = true,
                    _ => {}
                }
                fields
            },
        )
}

fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() > max_len {
        format!("{}...", &msg[..max_len.saturating_sub(3)])
    } else {
        msg.to_string()
    }
}

fn build_suggestion(missing_fields: &[&str], current_msg: &str, example_span: &str) -> String {
    match missing_fields {
        ["label", "help"] => format!(
            "Add 'label' and 'help' fields to provide context.\nExample:\nerror make {{\n\x20 \
             msg: \"{current_msg}\"\n\x20 label: {{\n\x20   text: \"describe what's wrong \
             here\"\n\x20   span: {example_span}\n\x20 }}\n\x20 help: \"explain how to fix this \
             issue\"\n}}"
        ),
        ["label"] => format!(
            "Add 'label' field to pinpoint the error location.\nExample:\nerror make {{\n\x20 \
             msg: \"{current_msg}\"\n\x20 label: {{\n\x20   text: \"describe what's wrong \
             here\"\n\x20   span: {example_span}\n\x20 }}\n\x20 help: ...\n}}"
        ),
        ["help"] => format!(
            "Add 'help' field to guide users toward a solution.\nExample:\nerror make {{\n\x20 \
             msg: \"{current_msg}\"\n\x20 label: ...\n\x20 help: \"explain how to fix this \
             issue\"\n}}"
        ),
        _ => String::new(),
    }
}

fn check_error_make_metadata(
    record: &[RecordItem],
    context: &LintContext,
    call_span: nu_protocol::Span,
) -> Option<Violation> {
    let fields = analyze_record_fields(record, context);

    if !fields.has_msg || fields.is_valid() {
        return None;
    }

    let missing_fields = fields.missing_fields();
    let missing_list = missing_fields.join(", ");

    let current_msg = extract_msg_value(record, context)
        .as_deref()
        .map_or_else(|| "<dynamic>".to_string(), |m| truncate_message(m, 40));

    let example_span = extract_first_function_parameter(context, call_span).map_or_else(
        || "$span".to_string(),
        |param| format!("(metadata ${param}).span"),
    );

    let suggestion = build_suggestion(&missing_fields, &current_msg, &example_span);

    Some(
        Violation::new_dynamic(
            "error_make_metadata",
            format!("error make call is missing metadata fields: {missing_list}"),
            call_span,
        )
        .with_suggestion_dynamic(suggestion),
    )
}

fn extract_record_from_expr(expr: &Expression) -> Option<&Vec<RecordItem>> {
    match &expr.expr {
        Expr::Record(record) => Some(record),
        Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
            Expr::Record(record) => Some(record),
            _ => None,
        },
        _ => None,
    }
}

fn check_error_make_call(call: &Call, context: &LintContext) -> Option<Violation> {
    call.is_call_to_command("error make", context)
        .then_some(())
        .and_then(|()| call.get_first_positional_arg())
        .and_then(extract_record_from_expr)
        .and_then(|record| check_error_make_metadata(record, context, call.span()))
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) => check_error_make_call(call, ctx).into_iter().collect(),
        _ => vec![],
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "error_make_metadata",
        "error make calls should include metadata fields like label and help for better error \
         context",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
