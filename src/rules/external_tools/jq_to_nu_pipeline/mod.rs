use nu_protocol::{
    Span, Type,
    ast::{Expr, Expression},
};

use crate::{
    LintLevel,
    ast::external::{CliSpec, ExternalInvocation, Flag, InputSource, literal_content},
    context::LintContext,
    dsl::jq,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const RAW_OUTPUT: Flag = Flag::switch('r', "raw-output");
const COMPACT_OUTPUT: Flag = Flag::switch('c', "compact-output");

static SPEC: CliSpec = CliSpec {
    flags: &[RAW_OUTPUT, COMPACT_OUTPUT],
    numeric_shorthand: None,
};

const NOTE: &str = "Nu parses JSON into tables and records, so field access and sorting work on \
                    the value directly. Missing fields raise an error instead of silently \
                    producing null.";

enum JsonInput<'a> {
    File(&'a Expression),
    Text,
    SerializedValue { serialization_start: usize },
}

pub struct JqTranslation<'a> {
    span: Span,
    input: JsonInput<'a>,
    conversion: jq::NuEquivalent,
}

impl<'a> JqTranslation<'a> {
    fn from_invocation(
        invocation: &ExternalInvocation<'a>,
        context: &'a LintContext,
    ) -> Option<Self> {
        let parsed = invocation.parse(&SPEC)?;
        let input = match (invocation.input, parsed.operands.as_slice()) {
            (InputSource::Leading, [_, file]) => JsonInput::File(file),
            (InputSource::Piped, [_]) => Self::piped_input(invocation, context)?,
            _ => return None,
        };
        let filter = parsed.operands.first()?;
        let conversion = match &filter.expr {
            Expr::StringInterpolation(parts) => jq::convert_interpolation(parts, context)?,
            _ => jq::convert(literal_content(filter)?)?,
        };
        Some(Self {
            span: invocation.span,
            input,
            conversion,
        })
    }

    fn piped_input(
        invocation: &ExternalInvocation<'a>,
        context: &'a LintContext,
    ) -> Option<JsonInput<'a>> {
        let previous = invocation.previous?;
        if invocation.previous_command_name(context) == Some("to json") {
            return Some(JsonInput::SerializedValue {
                serialization_start: previous.span.start,
            });
        }
        let produces_text =
            matches!(previous.expr, Expr::ExternalCall(..)) || previous.ty == Type::String;
        produces_text.then_some(JsonInput::Text)
    }

    fn to_fix(&self, context: &LintContext) -> Fix {
        let command = self.conversion.to_nu_command(context);
        let (span, replacement) = match self.input {
            JsonInput::File(file) => (
                self.span,
                format!(
                    "open --raw {} | from json | {command}",
                    context.expr_text(file)
                ),
            ),
            JsonInput::Text => (self.span, format!("from json | {command}")),
            JsonInput::SerializedValue {
                serialization_start,
            } => (Span::new(serialization_start, self.span.end), command),
        };
        Fix {
            explanation: format!("Query the parsed value with '{replacement}'").into(),
            replacements: vec![Replacement::new(span, replacement)],
        }
    }
}

struct JqToNuPipeline;

impl DetectFix for JqToNuPipeline {
    type FixInput<'a> = JqTranslation<'a>;

    fn id(&self) -> &'static str {
        "jq_to_nu_pipeline"
    }

    fn short_description(&self) -> &'static str {
        "Simple `jq` filter replaceable with Nushell pipeline"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/cookbook/jq_v_nushell.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["jq"])
            .iter()
            .filter_map(|invocation| {
                let translation = JqTranslation::from_invocation(invocation, context)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("'jq' filter with a Nu equivalent");
                Some((detection, translation))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, translation: &Self::FixInput<'_>) -> Option<Fix> {
        Some(translation.to_fix(context))
    }
}

pub static RULE: &dyn Rule = &JqToNuPipeline;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
