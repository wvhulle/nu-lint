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

    for param in &signature.required_positional {
        if let Some(var_id) = param.var_id {
            let var = context.working_set.get_variable(var_id);
            let param_span = var.declaration_span;

            if !param_span.has_inline_doc_comment(context) {
                violations.push(
                    Violation::new(format!(
                            "Positional parameter '{}' in main function is missing documentation \
                             comment",
                            param.name
                        ),
                        param_span,
                    )
                    .with_help(format!(
                        "Add a documentation comment after the parameter: {} # Description of {}",
                        param.name, param.name
                    )),
                );
            }
        }
    }

    for param in &signature.optional_positional {
        if let Some(var_id) = param.var_id {
            let var = context.working_set.get_variable(var_id);
            let param_span = var.declaration_span;

            if !param_span.has_inline_doc_comment(context) {
                violations.push(
                    Violation::new(format!(
                            "Optional positional parameter '{}' in main function is missing \
                             documentation comment",
                            param.name
                        ),
                        param_span,
                    )
                    .with_help(format!(
                        "Add a documentation comment after the parameter: {} # Description of {}",
                        param.name, param.name
                    )),
                );
            }
        }
    }

    if let Some(rest_param) = &signature.rest_positional
        && let Some(var_id) = rest_param.var_id
    {
        let var = context.working_set.get_variable(var_id);
        let param_span = var.declaration_span;

        if !param_span.has_inline_doc_comment(context) {
            violations.push(
                Violation::new(format!(
                        "Rest positional parameter '{}' in main function is missing documentation \
                         comment",
                        rest_param.name
                    ),
                    param_span,
                )
                .with_help(format!(
                    "Add a documentation comment after the parameter: ...{} # Description of {}",
                    rest_param.name, rest_param.name
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
        "main_positional_args_docs",
        "Positional parameters in main functions should have documentation comments",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/custom_commands.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
