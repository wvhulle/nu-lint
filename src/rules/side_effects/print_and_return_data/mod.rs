use nu_protocol::{
    Span, Type,
    ast::{Block, Call, Expr},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, span::SpanExt},
    context::LintContext,
    effect::{
        builtin::{BuiltinEffect, has_builtin_side_effect},
        external::{ExternEffect, has_external_side_effect},
    },
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn collect_stdout_print_spans(block: &Block, context: &LintContext) -> Vec<Span> {
    use nu_protocol::ast::Traverse;

    let mut print_spans = Vec::new();
    block.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr
                && call.is_call_to_command("print", context)
                && !call.has_named_flag("stderr")
            {
                return vec![call.head];
            }
            vec![]
        },
        &mut print_spans,
    );
    print_spans
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
            let cmd_name = head.span.source_code(context);
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

fn check_function_definition(call: &Call, context: &LintContext) -> Option<Detection> {
    let def = call.extract_function_definition(context)?;

    if def.name == "main" {
        return None;
    }

    let block = context.working_set.get_block(def.body);
    let print_spans = collect_stdout_print_spans(block, context);

    if print_spans.is_empty() || !returns_data(block, context) {
        return None;
    }

    let message = format!(
        "Function `{}` both prints to stdout and returns data, which pollutes pipelines",
        def.name
    );

    let suggestion = "Use `print -e` for stderr, separate into data/logging functions, or \
                      document the intentional mixing"
        .to_string();

    let mut violation = Detection::from_global_span(message, def.name_span)
        .with_primary_label("function with mixed output")
        .with_help(suggestion);

    for span in print_spans {
        violation = violation.with_extra_label("prints to stdout", span);
    }

    Some(violation)
}

struct PrintAndReturnData;

impl DetectFix for PrintAndReturnData {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "print_and_return_data"
    }

    fn explanation(&self) -> &'static str {
        "Functions should not both print to stdout and return data. This is confusing."
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let violations = context.detect(|expr, ctx| {
            if let Expr::Call(call) = &expr.expr
                && call.extract_function_definition(ctx).is_some()
            {
                return check_function_definition(call, ctx).into_iter().collect();
            }
            vec![]
        });
        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &PrintAndReturnData;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
