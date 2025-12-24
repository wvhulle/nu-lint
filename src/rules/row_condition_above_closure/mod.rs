use nu_protocol::ast::{Call, Expr, Expression};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

const fn is_stored_closure(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::Var(_) | Expr::FullCellPath(_))
}

fn extract_closure_parameter_name(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
) -> Option<String> {
    let block = context.working_set.get_block(block_id);

    if block.signature.required_positional.len() != 1 {
        return None;
    }

    let param = &block.signature.required_positional[0];
    let var_id = param.var_id?;

    let var = context.working_set.get_variable(var_id);
    Some(var.declaration_span.source_code(context).to_string())
}

fn generate_fix(
    closure_expr: &Expression,
    block_id: nu_protocol::BlockId,
    param_name: &str,
    context: &LintContext,
) -> Option<Fix> {
    let block = context.working_set.get_block(block_id);

    if block.pipelines.is_empty() {
        return None;
    }

    let block_span = block.span?;
    let block_text = block_span.source_code(context);

    let body_text = block_text
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(block_text)
        .trim();

    let closure_param_text = format!("|{param_name}|");
    let body_without_closure = body_text
        .strip_prefix(&closure_param_text)
        .unwrap_or(body_text)
        .trim();

    let param_pattern = format!("${param_name}");
    let fixed_text = body_without_closure.replace(&param_pattern, "$it");

    let explanation =
        format!("Replace closure parameter `${param_name}` with `$it` to use row condition syntax");

    Some(Fix::with_explanation(
        explanation,
        vec![Replacement::new(closure_expr.span, fixed_text)],
    ))
}

fn check_where_call(call: &Call, _expr: &Expression, context: &LintContext) -> Vec<Violation> {
    if call.get_call_name(context) != "where" {
        return vec![];
    }

    let Some(arg_expr) = call.get_first_positional_arg() else {
        return vec![];
    };

    if is_stored_closure(arg_expr) {
        return vec![];
    }

    let Expr::RowCondition(block_id) = &arg_expr.expr else {
        return vec![];
    };

    let Some(param_name) = extract_closure_parameter_name(*block_id, context) else {
        return vec![];
    };

    if param_name == "it" {
        return vec![];
    }

    let arg_text = arg_expr.span.source_code(context);
    let closure_param_syntax = format!("|{param_name}|");
    if !arg_text.contains(&closure_param_syntax) {
        return vec![];
    }

    let fix = generate_fix(arg_expr, *block_id, &param_name, context);

    let violation = Violation::new(
        "Use row condition with `$it` instead of closure for more concise code",
        arg_expr.span,
    )
    .with_primary_label("closure syntax")
    .with_extra_label("where command", call.span())
    .with_help(
        "Replace `where {|param| $param ...}` with `where $it ...` for simpler syntax. Row \
         conditions are more concise and idiomatic when you don't need to store the condition in \
         a variable.",
    );

    vec![match fix {
        Some(f) => violation.with_fix(f),
        None => violation,
    }]
}

fn check_expression(expr: &Expression, context: &LintContext) -> Vec<Violation> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    check_where_call(call, expr, context)
}

fn check(context: &LintContext) -> Vec<Violation> {
    use nu_protocol::ast::Traverse;

    let mut violations = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| check_expression(expr, context),
        &mut violations,
    );

    violations
}

pub const RULE: Rule = Rule::new(
    "row_condition_above_closure",
    "Prefer row conditions over closures in 'where' for conciseness",
    check,
    LintLevel::Hint,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/commands/docs/where.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
