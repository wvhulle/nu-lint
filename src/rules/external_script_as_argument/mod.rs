use nu_protocol::{
    SyntaxShape, VarId,
    ast::{Block, FindMapResult, Traverse},
};

use crate::{
    LintLevel,
    ast::{declaration::CustomCommandDef, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn contains_external_call_with_variable(
    block: &Block,
    var_id: VarId,
    context: &LintContext,
) -> bool {
    block
        .find_map(context.working_set, &|expr| {
            if expr.is_external_call_with_variable(var_id) {
                FindMapResult::Found(())
            } else {
                FindMapResult::Continue
            }
        })
        .is_some()
}

/// Check if a parameter is a string-like type that could be used as a script
/// path
const fn is_string_parameter(param: &nu_protocol::PositionalArg) -> bool {
    matches!(
        param.shape,
        SyntaxShape::String | SyntaxShape::Filepath | SyntaxShape::Any
    )
}

/// Create a violation for a parameter that's used as an external command
fn create_violation(
    param_name: &str,
    function_def: &CustomCommandDef,
    context: &LintContext,
) -> Detection {
    Detection::from_file_span(
        format!(
            "Function '{}' parameter '{param_name}' is used as an external command.",
            function_def.name
        ),
        function_def.declaration_span(context),
    )
    .with_primary_label("using script parameter")
}

struct ExternalScriptAsArgument;

impl DetectFix for ExternalScriptAsArgument {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "external_script_as_argument"
    }

    fn short_description(&self) -> &'static str {
        "Avoid passing external scripts as arguments to custom commands; define them as functions \
         instead"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/modules.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let detections = context
            .custom_commands()
            .iter()
            .flat_map(|def| {
                let function_block = context.working_set.get_block(def.body);

                function_block
                    .signature
                    .required_positional
                    .iter()
                    .chain(&function_block.signature.optional_positional)
                    .filter(|param| is_string_parameter(param))
                    .filter_map(|param| param.var_id.map(|var_id| (param, var_id)))
                    .filter(|(_, var_id)| {
                        contains_external_call_with_variable(function_block, *var_id, context)
                    })
                    .map(|(param, _)| create_violation(&param.name, def, context))
                    .collect::<Vec<_>>()
            })
            .collect();

        Self::no_fix(detections)
    }
}

pub static RULE: &dyn Rule = &ExternalScriptAsArgument;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
