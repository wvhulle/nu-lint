use std::collections::BTreeSet;

use nu_protocol::Span;

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    span: Span,
}

struct NothingOutsideFunctionSignature;

impl DetectFix for NothingOutsideFunctionSignature {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "nothing_outside_function_signature"
    }

    fn short_description(&self) -> &'static str {
        "The keyword 'nothing' should only appear in function signatures (as return type \
         annotations), not in function bodies or other expressions. Using 'nothing' outside of \
         signatures is a programming error."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/types.html#nothing")
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            r#"'nothing' is a type annotation keyword for function signatures. It indicates that a function returns no value.

To represent the absence of a value in a function body, use 'null' instead."#,
        )
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Error)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let signature_spans: Vec<Span> = context
            .custom_commands()
            .into_iter()
            .map(|cmd| cmd.signature_span)
            .collect();

        let results = context.detect_with_fix_data(|expr, context| {
            let expr_text = context.plain_text(expr.span).trim();

            if expr_text != "nothing" {
                return vec![];
            }

            let is_in_signature = signature_spans
                .iter()
                .any(|sig_span| sig_span.contains_span(expr.span));

            if is_in_signature {
                return vec![];
            }

            let detection = Detection::from_global_span(
                "The keyword 'nothing' should only be used in function signatures, not in \
                 expressions or function bodies",
                expr.span,
            )
            .with_primary_label("'nothing' used outside function signature");

            vec![(detection, FixData { span: expr.span })]
        });

        let mut seen = BTreeSet::new();
        results
            .into_iter()
            .filter(|(_, fix_data)| seen.insert((fix_data.span.start, fix_data.span.end)))
            .collect()
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            "Replace 'nothing' with 'null'",
            vec![Replacement::new(fix_data.span, "null".to_string())],
        ))
    }
}

pub static RULE: &dyn Rule = &NothingOutsideFunctionSignature;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
