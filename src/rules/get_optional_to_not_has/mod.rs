use nu_protocol::{
    Span,
    ast::{Call, Pipeline},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, pipeline::PipelineExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

struct FixData {
    span: Span,
    record_span: Span,
    key_span: Span,
}

fn is_is_empty(call: &Call, ctx: &LintContext) -> bool {
    call.is_call_to_command("is-empty", ctx)
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    pipeline
        .find_command_pairs(context, CallExt::is_get_optional, is_is_empty)
        .into_iter()
        .filter_map(|pair| {
            let get_idx = pair.first_index;
            if get_idx == 0 {
                return None;
            }

            let key_arg = pair.first.get_first_positional_arg()?;

            let record_span = Span::new(
                pipeline.elements[0].expr.span.start,
                pipeline.elements[get_idx - 1].expr.span.end,
            );

            // Span covering the entire pattern: $record | get -i $key | is-empty
            let full_span = Span::new(pipeline.elements[0].expr.span.start, pair.span.end);

            let detection = Detection::from_global_span(
                "Use 'not-has' operator instead of 'get -o | is-empty' for record key membership",
                full_span,
            )
            .with_primary_label("non-idiomatic key check")
            .with_extra_label("get -o call", pair.first.span())
            .with_extra_label("is-empty call", pair.second.span());

            Some((
                detection,
                FixData {
                    span: full_span,
                    record_span,
                    key_span: key_arg.span,
                },
            ))
        })
        .collect()
}

struct GetOptionalToNotHas;

impl DetectFix for GetOptionalToNotHas {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "get_optional_to_not_has"
    }

    fn short_description(&self) -> &'static str {
        "Simplify `get -o | is-empty` to `not-has`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "The pattern `$record | get -o $key | is-empty` can be written more idiomatically as \
             `$record not-has $key`. The 'not-has' operator directly checks if a key is absent \
             from a record.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/operators.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let record_text = context.span_text(fix_data.record_span).trim();
        let key_text = context.span_text(fix_data.key_span).trim();

        let replacement = format!("{record_text} not-has {key_text}");

        Some(Fix {
            explanation: "Replace with 'not-has' operator".into(),
            replacements: vec![Replacement::new(fix_data.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &GetOptionalToNotHas;
