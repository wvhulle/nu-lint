use nu_protocol::{
    Type,
    ast::{Block, Call, Expr, Expression},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, effect::is_side_effect_only},
    context::LintContext,
    rule::Rule,
    violation::RuleViolation,
};

fn has_print_call(block: &Block, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let mut print_calls = Vec::new();
    block.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr
                && call.is_call_to_command("print", context)
                && !call.has_named_flag("stderr")
            {
                return vec![()];
            }
            vec![]
        },
        &mut print_calls,
    );
    !print_calls.is_empty()
}

fn function_returns_data(block: &Block, context: &LintContext) -> bool {
    let output_type = block.infer_output_type(context);

    // A function returns data if:
    // 1. It has a specific output type (not Nothing or Any)
    // 2. OR it has Any output type but doesn't end with a side-effect-only command
    match output_type {
        Type::Nothing => false,
        Type::Any => {
            // Check if the last expression is likely to return data
            // by looking at the last pipeline
            block.pipelines.last().is_some_and(|pipeline| {
                pipeline
                    .elements
                    .last()
                    .is_some_and(|element| !is_side_effect_only_command(&element.expr, context))
            })
        }
        _ => true,
    }
}

fn is_side_effect_only_command(expr: &Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let cmd_name = call.get_call_name(context);
            is_side_effect_only(&cmd_name)
        }
        _ => false,
    }
}

fn check_function_definition(call: &Call, context: &LintContext) -> Option<RuleViolation> {
    let (block_id, func_name) = call.extract_function_definition(context)?;

    // Skip main function as it often combines output and side effects
    if func_name == "main" {
        return None;
    }

    let block = context.working_set.get_block(block_id);

    let has_print = has_print_call(block, context);
    let returns_data = function_returns_data(block, context);

    if !has_print || !returns_data {
        return None;
    }

    let name_span = call.get_positional_arg(0)?.span;

    let message = format!(
        "Function `{func_name}` both prints to stdout and returns data, which pollutes pipelines"
    );

    let suggestion = "Use `print -e` for stderr, separate into data/logging functions, or \
                      document the intentional mixing"
        .to_string();

    Some(
        RuleViolation::new_dynamic("print_and_return_data", message, name_span)
            .with_suggestion_dynamic(suggestion),
    )
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr
            && call.extract_function_definition(ctx).is_some()
        {
            return check_function_definition(call, ctx).into_iter().collect();
        }
        vec![]
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "print_and_return_data",
        LintLevel::Warn,
        "Functions should not both print to stdout and return data",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
