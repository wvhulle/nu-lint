use nu_protocol::ast::{Call, Expr};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_main_function(call: &Call, context: &LintContext) -> Vec<Detection> {
    let (_func_name, _name_span) = match call.extract_declaration_name(context) {
        Some((name, span)) if name == "main" => (name, span),
        _ => return vec![],
    };

    let Some(def) = call.extract_function_definition(context) else {
        return vec![];
    };

    let block = context.working_set.get_block(def.body);
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
                Detection::from_global_span(
                    format!(
                        "Named parameter '{flag_name}' in main function is missing documentation \
                         comment"
                    ),
                    flag_span,
                )
                .with_primary_label("undocumented flag")
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

struct MainNamedArgsDocs;

impl DetectFix for MainNamedArgsDocs {
    type FixInput = ();

    fn id(&self) -> &'static str {
        "main_named_args_docs"
    }

    fn explanation(&self) -> &'static str {
        "Named parameters (flags) in main functions should have documentation comments"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#flags")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        Self::no_fix(context.detect(|expr, ctx| {
            if let Expr::Call(call) = &expr.expr {
                let decl_name = call.get_call_name(ctx);
                if decl_name == "def" {
                    return check_main_function(call, ctx);
                }
            }
            vec![]
        }))
    }
}

pub static RULE: &dyn Rule = &MainNamedArgsDocs;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
