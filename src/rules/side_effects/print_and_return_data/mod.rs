use nu_protocol::{
    Type,
    ast::{Block, Call, Expr},
};

use crate::{
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    effect::{
        builtin::{BuiltinEffect, has_builtin_side_effect},
        external::{ExternEffect, has_external_side_effect},
    },
    rule::Rule,
    violation::Violation,
};

fn has_stdout_print(block: &Block, context: &LintContext) -> bool {
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

fn last_command_produces_output(block: &Block, context: &LintContext) -> bool {
    let Some(last_pipeline) = block.pipelines.last() else {
        return false;
    };
    let Some(last_element) = last_pipeline.elements.last() else {
        return false;
    };

    match &last_element.expr.expr {
        Expr::Call(call) => {
            // If it explicitly prints to stdout, we treat that as output for pollution
            // purposes
            let cmd_name = call.get_call_name(context);
            if has_builtin_side_effect(&cmd_name, BuiltinEffect::PrintToStdout, context, call) {
                return true;
            }

            // Use the signature to decide if it produces data. Any -> assume output unless
            // signature maps only to Nothing.
            let decl = context.working_set.get_decl(call.decl_id);
            let sig = decl.signature();
            // If every mapping returns Nothing, then no output.
            sig.input_output_types
                .iter()
                .any(|(_in, out)| !matches!(out, nu_protocol::Type::Nothing))
        }
        Expr::ExternalCall(head, args) => {
            // Extract external command name span text
            let cmd_name = &context.source[head.span.start..head.span.end];
            // If external side effect registry marks command as NoDataInStdout, treat as no
            // output
            !has_external_side_effect(cmd_name, ExternEffect::NoDataInStdout, context, args)
        }
        _ => false,
    }
}

fn returns_data(block: &Block, context: &LintContext) -> bool {
    let output_type = block.infer_output_type(context);

    match output_type {
        Type::Nothing => false,
        Type::Any => last_command_produces_output(block, context),
        _ => true,
    }
}

fn check_function_definition(call: &Call, context: &LintContext) -> Option<Violation> {
    let (block_id, func_name) = call.extract_function_definition(context)?;

    if func_name == "main" {
        return None;
    }

    let block = context.working_set.get_block(block_id);

    if !has_stdout_print(block, context) || !returns_data(block, context) {
        return None;
    }

    let name_span = call.get_positional_arg(0)?.span;

    let message = format!(
        "Function `{func_name}` both prints to stdout and returns data, which pollutes pipelines"
    );

    let suggestion = "Use `print -e` for stderr, separate into data/logging functions, or \
                      document the intentional mixing"
        .to_string();

    Some(Violation::new("print_and_return_data", message, name_span).with_help(suggestion))
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr
            && call.extract_function_definition(ctx).is_some()
        {
            return check_function_definition(call, ctx).into_iter().collect();
        }
        vec![]
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "print_and_return_data",
        "Functions should not both print to stdout and return data",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
