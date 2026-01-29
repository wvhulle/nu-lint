use nu_protocol::ast::{Expr, ExternalArgument, Traverse};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct TailFixData<'a> {
    count: Option<&'a str>,
    filename: Option<&'a str>,
    follow: bool,
    expr_span: nu_protocol::Span,
}

struct UseBuiltinTail;

impl DetectFix for UseBuiltinTail {
    type FixInput<'a> = TailFixData<'a>;

    fn id(&self) -> &'static str {
        "tail_to_last"
    }

    fn short_description(&self) -> &'static str {
        "`tail` replaceable with `last`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/last.html")
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

                let cmd_text = context.expr_text(head);
                if cmd_text != "tail" {
                    return vec![];
                }

                let args_with_spans: Vec<_> =
                    args.iter()
                        .map(|arg| {
                            let span = match arg {
                                ExternalArgument::Regular(expr)
                                | ExternalArgument::Spread(expr) => expr.span,
                            };
                            (context.span_text(span), span)
                        })
                        .collect();

                let follow = args_with_spans
                    .iter()
                    .any(|(text, _)| *text == "-f" || *text == "-F" || *text == "--follow");

                let count = args_with_spans
                    .iter()
                    .find(|(text, _)| {
                        text.starts_with('-')
                            && text.len() > 1
                            && *text != "-f"
                            && *text != "-F"
                            && *text != "--follow"
                    })
                    .map(|(text, _)| &text[1..]);

                let filename = args_with_spans
                    .iter()
                    .find(|(text, _)| !text.starts_with('-'))
                    .map(|(text, _)| *text);

                let detection = args_with_spans.iter().fold(
                    Detection::from_global_span("Use 'last N' to get the last N items", head.span)
                        .with_primary_label("external 'tail'"),
                    |det, (text, span)| {
                        if text.starts_with('-') && text.len() > 1 {
                            det.with_extra_label("line count", *span)
                        } else {
                            det.with_extra_label("file", *span)
                        }
                    },
                );

                let fix_data = TailFixData {
                    count,
                    filename,
                    follow,
                    expr_span: expr.span,
                };

                vec![(detection, fix_data)]
            },
            &mut results,
        );

        results
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        if fix_data.follow {
            let file = fix_data.filename.unwrap_or("file");
            let count = fix_data.count.unwrap_or("20");
            let replacement =
                format!("watch {file} {{ open --raw {file} | lines | last {count} }}");
            let description = "Use 'watch' to monitor file changes. Nu's watch executes a closure \
                               when the file changes, similar to 'tail -f'. Note: this is \
                               event-based, not continuous streaming.";

            return Some(Fix {
                explanation: description.into(),
                replacements: vec![Replacement {
                    span: fix_data.expr_span.into(),
                    replacement_text: replacement.into(),
                }],
            });
        }

        let count = fix_data.count.unwrap_or("10");

        let replacement = fix_data.filename.map_or_else(
            || format!("last {count}"),
            |file| format!("open {file} | lines | last {count}"),
        );

        let description = "Use 'last' with cleaner syntax: 'last N' instead of 'tail -N'";

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinTail;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
