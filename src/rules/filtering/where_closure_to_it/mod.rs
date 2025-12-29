use nu_protocol::ast::{Call, Expr, Expression, Traverse};

use crate::{
    LintLevel,
    ast::{call::CallExt, string::strip_block_braces},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct RowConditionFixData {
    closure_span: nu_protocol::Span,
    param_decl_span: nu_protocol::Span,
    param_name: String,
}

const fn is_stored_closure(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::Var(_) | Expr::FullCellPath(_))
}

fn extract_closure_parameter(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
) -> Option<(String, nu_protocol::Span)> {
    let block = context.working_set.get_block(block_id);

    if block.signature.required_positional.len() != 1 {
        return None;
    }

    let param = &block.signature.required_positional[0];
    let var_id = param.var_id?;

    let var = context.working_set.get_variable(var_id);
    let param_name = context.get_span_text(var.declaration_span).to_string();

    Some((param_name, var.declaration_span))
}

fn find_param_decl_span_in_source(
    block_span: nu_protocol::Span,
    param_decl_span: nu_protocol::Span,
    context: &LintContext,
) -> Option<nu_protocol::Span> {
    let block_text = context.get_span_text(block_span);
    let param_text = context.get_span_text(param_decl_span);

    let param_syntax = format!("|{param_text}|");
    let offset = block_text.find(&param_syntax)?;

    let start = block_span.start + offset;
    let end = start + param_syntax.len();

    Some(nu_protocol::Span::new(start, end))
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

    let block = context.working_set.get_block(*block_id);
    let Some(block_span) = block.span else {
        return vec![];
    };

    let Some((param_name, param_var_span)) = extract_closure_parameter(*block_id, context) else {
        return vec![];
    };

    if param_name == "it" {
        return vec![];
    }

    let arg_text = context.get_span_text(arg_expr.span);
    let closure_param_syntax = format!("|{param_name}|");
    if !arg_text.contains(&closure_param_syntax) {
        return vec![];
    }

    let Some(param_decl_span) = find_param_decl_span_in_source(block_span, param_var_span, context)
    else {
        return vec![];
    };

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
        param_decl_span,
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

struct WhereClosureToIt;

impl DetectFix for WhereClosureToIt {
    type FixInput<'a> = RowConditionFixData;

    fn id(&self) -> &'static str {
        "where_closure_to_it"
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

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let closure_text = context.get_span_text(fix_data.closure_span);
        let param_decl_text = context.get_span_text(fix_data.param_decl_span);

        let body_text = strip_block_braces(closure_text);

        let body_without_param = body_text
            .strip_prefix(param_decl_text)
            .unwrap_or(body_text)
            .trim();

        let param_pattern = format!("${}", fix_data.param_name);
        let fixed_text = body_without_param.replace(&param_pattern, "$it");

        Some(Fix::with_explanation(
            format!(
                "Replace closure parameter ${} with row condition using $it",
                fix_data.param_name
            ),
            vec![Replacement::new(fix_data.closure_span, fixed_text)],
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
