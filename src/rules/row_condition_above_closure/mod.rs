use nu_protocol::ast::{Call, Expr, Expression, Traverse};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct RowConditionFixData {
    closure_span: nu_protocol::Span,
    fixed_text: Option<String>,
    param_name: String,
}

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

fn check_where_call(
    call: &Call,
    _expr: &Expression,
    context: &LintContext,
) -> Vec<(Detection, RowConditionFixData)> {
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

    let fixed_text = generate_fix(arg_expr, *block_id, &param_name, context).and_then(|f| {
        f.replacements
            .first()
            .map(|r| r.replacement_text.to_string())
    });

    let violation = Detection::from_global_span(
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

    let fix_data = RowConditionFixData {
        closure_span: arg_expr.span,
        fixed_text,
        param_name,
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

    check_where_call(call, expr, context)
}

struct RowConditionAboveClosure;

impl DetectFix for RowConditionAboveClosure {
    type FixInput<'a> = RowConditionFixData;

    fn id(&self) -> &'static str {
        "row_condition_above_closure"
    }

    fn explanation(&self) -> &'static str {
        "Prefer row conditions over closures in 'where' for conciseness"
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

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        fix_data.fixed_text.as_ref().map(|text| {
            Fix::with_explanation(
                format!(
                    "Replace closure parameter ${} with row condition using $it",
                    fix_data.param_name
                ),
                vec![Replacement::new(fix_data.closure_span, text.clone())],
            )
        })
    }
}

pub static RULE: &dyn Rule = &RowConditionAboveClosure;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
