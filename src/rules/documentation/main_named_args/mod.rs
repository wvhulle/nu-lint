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
                .with_primary_label("undocumented flag"),
            );
        }
    }

    violations
}

struct MainNamedArgsDocs;

impl DetectFix for MainNamedArgsDocs {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "main_named_args_docs"
    }

    fn short_description(&self) -> &'static str {
        "Missing docs on main flag parameter"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#flags")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
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

pub static RULE: &dyn Rule = &MainNamedArgsDocs;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
