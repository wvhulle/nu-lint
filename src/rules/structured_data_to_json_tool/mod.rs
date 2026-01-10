use nu_protocol::{Span, Type, ast::PipelineElement};

use crate::{
    Fix, Replacement,
    config::LintLevel,
    context::LintContext,
    format_conversions::{ConversionSpec, check_all_pipelines},
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct StructuredDataToJsonTool;

impl DetectFix for StructuredDataToJsonTool {
    type FixInput<'a> = Span;

    fn id(&self) -> &'static str {
        "structured_data_to_json_tool"
    }

    fn short_description(&self) -> &'static str {
        "Don't pipe structured data directly into JSON tools like jq without converting to JSON \
         first."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/pipelines.html#external-commands")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let spec = ConversionSpec {
            matches_command: &|cmd| matches!(cmd, "jq" | "json_pp" | "jsonlint"),
            matches_type: &|ty| matches!(ty, Type::Table(_) | Type::Record(_) | Type::List(_)),
        };

        check_all_pipelines(context, &spec, create_violation)
    }

    fn fix(&self, context: &LintContext, data_span: &Self::FixInput<'_>) -> Option<Fix> {
        let original_text = context.plain_text(*data_span);
        let new_text = format!("{original_text} | to json");

        Some(Fix::with_explanation(
            "Add 'to json' to convert to JSON before piping to JSON tool",
            vec![Replacement::new(*data_span, new_text)],
        ))
    }
}

fn create_violation(
    ty: &Type,
    cmd_name: &str,
    left: &PipelineElement,
    right: &PipelineElement,
) -> (Detection, Span) {
    let type_name = ty.to_string();
    let message =
        format!("Piping {type_name} into JSON tool '{cmd_name}' requires 'to json' conversion");

    let detection = Detection::from_global_span(message, right.expr.span)
        .with_extra_label(format!("{type_name} output"), left.expr.span)
        .with_extra_label("JSON tool", right.expr.span);

    (detection, left.expr.span)
}

pub static RULE: &dyn Rule = &StructuredDataToJsonTool;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
