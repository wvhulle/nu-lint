use nu_protocol::{Span, Type};

use crate::{
    Fix, Replacement,
    ast::{expression::ExpressionExt, external::ExternalInvocation},
    config::LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

const JSON_TOOLS: &[&str] = &["jq", "json_pp", "jsonlint", "gron", "fx", "jless", "yq"];
const CSV_TOOLS: &[&str] = &["csvlook", "csvstat", "csvcut", "csvgrep", "xsv", "qsv"];

#[derive(Clone, Copy)]
enum TextFormat {
    Json,
    Csv,
}

impl TextFormat {
    fn for_tool(name: &str) -> Self {
        if CSV_TOOLS.contains(&name) {
            Self::Csv
        } else {
            Self::Json
        }
    }

    fn accepts(self, ty: &Type) -> bool {
        match (self, ty) {
            (Self::Json, Type::Table(_) | Type::Record(_) | Type::List(_))
            | (Self::Csv, Type::Table(_)) => true,
            (Self::Csv, Type::List(item)) => matches!(**item, Type::Record(_)),
            _ => false,
        }
    }

    const fn command(self) -> &'static str {
        match self {
            Self::Json => "to json",
            Self::Csv => "to csv",
        }
    }
}

pub struct MissingSerialization {
    data_span: Span,
    data_type: Type,
    format: TextFormat,
}

fn missing_serialization(
    invocation: &ExternalInvocation,
    context: &LintContext,
) -> Option<MissingSerialization> {
    let data = invocation.previous?;
    let format = TextFormat::for_tool(invocation.name);
    let data_type = data.infer_output_type(context)?;
    format.accepts(&data_type).then_some(MissingSerialization {
        data_span: data.span,
        data_type,
        format,
    })
}

fn detection(invocation: &ExternalInvocation, missing: &MissingSerialization) -> Detection {
    let data_type = &missing.data_type;
    let message = format!(
        "Nu renders {data_type} as a table when piping it to '{}'; serialize it with '{}' first",
        invocation.name,
        missing.format.command()
    );
    Detection::from_global_span(message, invocation.span)
        .with_extra_label(format!("{data_type} output"), missing.data_span)
}

struct SerializeDataForExternal;

impl DetectFix for SerializeDataForExternal {
    type FixInput<'a> = MissingSerialization;

    fn id(&self) -> &'static str {
        "serialize_data_for_external"
    }

    fn short_description(&self) -> &'static str {
        "Structured data piped to a JSON or CSV tool without `to json` or `to csv`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/pipelines.html#external-commands")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let tools: Vec<&str> = JSON_TOOLS.iter().chain(CSV_TOOLS).copied().collect();
        context
            .external_invocations(&tools)
            .iter()
            .filter_map(|invocation| {
                let missing = missing_serialization(invocation, context)?;
                Some((detection(invocation, &missing), missing))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, missing: &Self::FixInput<'_>) -> Option<Fix> {
        let command = missing.format.command();
        Some(Fix {
            explanation: format!("Add '{command}' before the external tool").into(),
            replacements: vec![Replacement::new(
                missing.data_span,
                format!("{} | {command}", context.span_text(missing.data_span)),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &SerializeDataForExternal;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
