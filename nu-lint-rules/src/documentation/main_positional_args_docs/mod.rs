use nu_protocol::{
    PositionalArg,
    ast::{Call, Expr},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_positional_param(
    param: &PositionalArg,
    param_type: &str,
    context: &LintContext,
) -> Option<Detection> {
    let var_id = param.var_id?;
    let var = context.working_set.get_variable(var_id);
    let param_span = var.declaration_span;

    if param_span.has_inline_doc_comment(context) {
        return None;
    }

    Some(
        Detection::from_global_span(
            format!(
                "{} parameter '{}' in main function is missing documentation comment",
                param_type, param.name
            ),
            param_span,
        )
        .with_primary_label(format!(
            "undocumented {} parameter",
            param_type.to_lowercase()
        )),
    )
}

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
        if let Some(detection) = check_positional_param(param, "Positional", context) {
            violations.push(detection);
        }
    }

    for param in &signature.optional_positional {
        if let Some(detection) = check_positional_param(param, "Optional positional", context) {
            violations.push(detection);
        }
    }

    if let Some(rest_param) = &signature.rest_positional
        && let Some(detection) = check_positional_param(rest_param, "Rest positional", context)
    {
        violations.push(detection);
    }

    violations
}

struct MainPositionalArgsDocs;

impl DetectFix for MainPositionalArgsDocs {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "main_positional_args_docs"
    }

    fn short_description(&self) -> &'static str {
        "Missing docs on main positional parameter"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some("Add a documentation comment after the parameter: <param> # Description of <param>")
    }

    fn source_link(&self) -> Option<&'static str> {
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
