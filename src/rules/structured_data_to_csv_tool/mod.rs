use nu_protocol::{Span, Type, ast::PipelineElement};

use crate::{
    Fix, Replacement,
    config::LintLevel,
    context::LintContext,
    format_conversions::{ConversionSpec, check_all_pipelines},
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct StructuredDataToCsvTool;

impl DetectFix for StructuredDataToCsvTool {
    type FixInput<'a> = Span;

    fn id(&self) -> &'static str {
        "structured_data_to_csv_tool"
    }

    fn short_description(&self) -> &'static str {
        "Don't pipe tables directly into CSV tools without converting to CSV first."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/pipelines.html#external-commands")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let spec = ConversionSpec {
            matches_command: &|cmd| matches!(cmd, "csvlook" | "csvstat" | "csvcut" | "csvgrep"),
            matches_type: &|ty| matches!(ty, Type::Table(_)),
        };

        check_all_pipelines(context, &spec, create_violation)
    }

    fn fix(&self, context: &LintContext, data_span: &Self::FixInput<'_>) -> Option<Fix> {
        let original_text = context.span_text(*data_span);
        let new_text = format!("{original_text} | to csv");

        Some(Fix::with_explanation(
            "Add 'to csv' to convert to CSV before piping to CSV tool",
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
        format!("Piping {type_name} into CSV tool '{cmd_name}' requires 'to csv' conversion");

    let detection = Detection::from_global_span(message, right.expr.span)
        .with_extra_label(format!("{type_name} output"), left.expr.span)
        .with_extra_label("CSV tool", right.expr.span);

    (detection, left.expr.span)
}

pub static RULE: &dyn Rule = &StructuredDataToCsvTool;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
