use nu_protocol::ast::{Expr, ExternalArgument, Traverse};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct HeadFixData<'a> {
    count: Option<&'a str>,
    filename: Option<&'a str>,
    expr_span: nu_protocol::Span,
}

struct UseBuiltinHead;

impl DetectFix for UseBuiltinHead {
    type FixInput<'a> = HeadFixData<'a>;

    fn id(&self) -> &'static str {
        "head_to_first"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'first' command instead of 'head' for cleaner syntax"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/first.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut results = Vec::new();

        context.ast.flat_map(
            context.working_set,
            &|expr| {
                let Expr::ExternalCall(head, args) = &expr.expr else {
                    return vec![];
                };

                let cmd_text = context.plain_text(head.span);
                if cmd_text != "head" {
                    return vec![];
                }

                let args_with_spans: Vec<_> =
                    args.iter()
                        .map(|arg| {
                            let span = match arg {
                                ExternalArgument::Regular(expr)
                                | ExternalArgument::Spread(expr) => expr.span,
                            };
                            (context.plain_text(span), span)
                        })
                        .collect();

                let count = args_with_spans
                    .iter()
                    .find(|(text, _)| text.starts_with('-') && text.len() > 1)
                    .map(|(text, _)| &text[1..]);

                let filename = args_with_spans
                    .iter()
                    .find(|(text, _)| !text.starts_with('-'))
                    .map(|(text, _)| *text);

                let detection = args_with_spans.iter().fold(
                    Detection::from_global_span(
                        "Use 'first N' to get the first N items",
                        head.span,
                    )
                    .with_primary_label("external 'head'"),
                    |det, (text, span)| {
                        if text.starts_with('-') && text.len() > 1 {
                            det.with_extra_label("line count", *span)
                        } else {
                            det.with_extra_label("file", *span)
                        }
                    },
                );

                let fix_data = HeadFixData {
                    count,
                    filename,
                    expr_span: expr.span,
                };

                vec![(detection, fix_data)]
            },
            &mut results,
        );

        results
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let count = fix_data.count.unwrap_or("10");

        let replacement = fix_data.filename.map_or_else(
            || format!("first {count}"),
            |file| format!("open {file} | lines | first {count}"),
        );

        let description = "Use 'first' with cleaner syntax: 'first N' instead of 'head -N'";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinHead;

#[cfg(test)]
mod tests;
