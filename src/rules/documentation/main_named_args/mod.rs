use nu_protocol::ast::{Call, Expr};

use crate::{
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

fn check_main_function(call: &Call, context: &LintContext) -> Vec<Violation> {
    let (_func_name, _name_span) = match call.extract_declaration_name(context) {
        Some((name, span)) if name == "main" => (name, span),
        _ => return vec![],
    };

    let Some((block_id, _)) = call.extract_function_definition(context) else {
        return vec![];
    };

    let block = context.working_set.get_block(block_id);
    let signature = &block.signature;

    let mut violations = Vec::new();

    for flag in &signature.named {
        let Some(var_id) = flag.var_id else {
            continue;
        };

        let var = context.working_set.get_variable(var_id);
        let flag_span = var.declaration_span;

        if !flag_span.has_inline_doc_comment(context) {
            let flag_name = flag.short.map_or_else(
                || format!("--{}", flag.long),
                |short| format!("--{} (-{})", flag.long, short),
            );

            violations.push(
                Violation::new(format!(
                        "Named parameter '{flag_name}' in main function is missing documentation \
                         comment"
                    ),
                    flag_span,
                )
                .with_help(format!(
                    "Add a documentation comment after the parameter: {flag_name} # Description \
                     of {}",
                    flag.long
                )),
            );
        }
    }

    violations
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr {
            let decl_name = call.get_call_name(ctx);
            if decl_name == "def" {
                return check_main_function(call, ctx);
            }
        }
        vec![]
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "main_named_args_docs",
        "Named parameters (flags) in main functions should have documentation comments",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/custom_commands.html#flags")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
