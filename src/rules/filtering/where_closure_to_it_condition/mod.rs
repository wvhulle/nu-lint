use nu_protocol::ast::{Call, Expr, Expression, Traverse};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct RowConditionFixData {
    param_decl_span: nu_protocol::Span,
    param_name: String,
    var_usage_spans: Vec<nu_protocol::Span>,
}

const fn is_stored_closure(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::Var(_) | Expr::FullCellPath(_))
}

fn extract_closure_parameter(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
) -> Option<(String, nu_protocol::Span, nu_protocol::VarId)> {
    let block = context.working_set.get_block(block_id);

    if block.signature.required_positional.len() != 1 {
        return None;
    }

    let param = &block.signature.required_positional[0];
    let var_id = param.var_id?;

    let var = context.working_set.get_variable(var_id);
    let param_name = context.plain_text(var.declaration_span).to_string();

    Some((param_name, var.declaration_span, var_id))
}

fn find_param_decl_span_in_source(
    block_span: nu_protocol::Span,
    param_decl_span: nu_protocol::Span,
    context: &LintContext,
) -> Option<nu_protocol::Span> {
    let block_text = context.plain_text(block_span);
    let param_text = context.plain_text(param_decl_span);

    let param_syntax = format!("|{param_text}|");
    let offset = block_text.find(&param_syntax)?;

    let start = block_span.start + offset;
    let end = start + param_syntax.len();

    Some(nu_protocol::Span::new(start, end))
}

fn check_filter_command_call(
    call: &Call,
    _expr: &Expression,
    context: &LintContext,
) -> Vec<(Detection, RowConditionFixData)> {
    let command_name = call.get_call_name(context);
    if command_name != "where" && command_name != "filter" {
        return vec![];
    }

    let Some(arg_expr) = call.get_first_positional_arg() else {
        return vec![];
    };

    if is_stored_closure(arg_expr) {
        return vec![];
    }

    let block_id = match &arg_expr.expr {
        Expr::RowCondition(id) | Expr::Closure(id) => *id,
        _ => return vec![],
    };

    let block = context.working_set.get_block(block_id);
    let Some(block_span) = block.span else {
        return vec![];
    };

    let Some((param_name, param_var_span, var_id)) = extract_closure_parameter(block_id, context)
    else {
        return vec![];
    };

    if param_name == "it" {
        return vec![];
    }

    let arg_text = context.plain_text(arg_expr.span);
    let closure_param_syntax = format!("|{param_name}|");
    if !arg_text.contains(&closure_param_syntax) {
        return vec![];
    }

    let Some(param_decl_span) = find_param_decl_span_in_source(block_span, param_var_span, context)
    else {
        return vec![];
    };

    let var_usage_spans = block.find_var_usage_spans(var_id, context, |_, _, _| true);

    let violation = Detection::from_global_span(
        "Use `$it` instead of closure parameter for more concise code",
        arg_expr.span,
    )
    .with_primary_label("closure with named parameter")
    .with_extra_label(format!("{command_name} command"), call.span())
    .with_help(format!(
        "Replace `{command_name} {{|param| $param ...}}` with `{command_name} {{|it| $it ...}}` \
         or `{command_name} {{$it ...}}` for simpler syntax. Using `$it` is more concise and \
         idiomatic when you don't need to store the closure in a variable.",
    ));

    let fix_data = RowConditionFixData {
        param_decl_span,
        param_name,
        var_usage_spans,
    };

    vec![(violation, fix_data)]
}

fn check_expression(
    expr: &Expression,
    context: &LintContext,
) -> Vec<(Detection, RowConditionFixData)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    check_filter_command_call(call, expr, context)
}

struct WhereClosureToIt;

impl DetectFix for WhereClosureToIt {
    type FixInput<'a> = RowConditionFixData;

    fn id(&self) -> &'static str {
        "where_or_filter_closure_to_it_row_condition"
    }

    fn explanation(&self) -> &'static str {
        "Prefer `$it` over named closure parameters in 'where' and 'filter' for conciseness"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/where.html")
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

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let mut replacements = Vec::new();

        replacements.push(Replacement::new(fix_data.param_decl_span, String::new()));

        let mut spans: Vec<_> = fix_data.var_usage_spans.clone();
        spans.sort_by_key(|s| s.start);
        spans.dedup();

        let filtered_spans: Vec<_> = spans
            .iter()
            .filter(|span| {
                !spans
                    .iter()
                    .any(|other| other.start == span.start && other.end > span.end)
            })
            .collect();

        for var_span in filtered_spans {
            let var_text = context.plain_text(*var_span);
            let replacement = if var_text.starts_with('$') {
                format!("$it{}", &var_text[1 + fix_data.param_name.len()..])
            } else {
                format!("it{}", &var_text[fix_data.param_name.len()..])
            };
            replacements.push(Replacement::new(*var_span, replacement));
        }

        Some(Fix::with_explanation(
            format!(
                "Replace closure parameter ${} with row condition using $it",
                fix_data.param_name
            ),
            replacements,
        ))
    }
}

pub static RULE: &dyn Rule = &WhereClosureToIt;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
