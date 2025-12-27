use nu_protocol::ast::{Call, Expr};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_main_function(call: &Call, context: &LintContext) -> Vec<Detection> {
    let Some(def) = call.custom_command_def(context) else {
        return vec![];
    };

    if !def.is_main() {
        return vec![];
    }

    let block = context.working_set.get_block(def.body);
    let signature = &block.signature;

    let mut violations = Vec::new();

    for param in &signature.required_positional {
        if let Some(var_id) = param.var_id {
            let var = context.working_set.get_variable(var_id);
            let param_span = var.declaration_span;

            if !param_span.has_inline_doc_comment(context) {
                violations.push(
                    Detection::from_global_span(
                        format!(
                            "Positional parameter '{}' in main function is missing documentation \
                             comment",
                            param.name
                        ),
                        param_span,
                    )
                    .with_primary_label("undocumented parameter")
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
                    Detection::from_global_span(
                        format!(
                            "Optional positional parameter '{}' in main function is missing \
                             documentation comment",
                            param.name
                        ),
                        param_span,
                    )
                    .with_primary_label("undocumented optional parameter")
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
                Detection::from_global_span(
                    format!(
                        "Rest positional parameter '{}' in main function is missing documentation \
                         comment",
                        rest_param.name
                    ),
                    param_span,
                )
                .with_primary_label("undocumented rest parameter")
                .with_help(format!(
                    "Add a documentation comment after the parameter: ...{} # Description of {}",
                    rest_param.name, rest_param.name
                )),
            );
        }
    }

    violations
}

struct MainPositionalArgsDocs;

impl DetectFix for MainPositionalArgsDocs {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "main_positional_args_docs"
    }

    fn explanation(&self) -> &'static str {
        "Positional parameters in main functions should have documentation comments"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, ctx| {
            if let Expr::Call(call) = &expr.expr
                && let Some(func_def) = call.custom_command_def(ctx)
                && !func_def.is_exported()
            {
                return check_main_function(call, ctx);
            }
            vec![]
        }))
    }
}

pub static RULE: &dyn Rule = &MainPositionalArgsDocs;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
