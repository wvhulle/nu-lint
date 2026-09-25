use nu_protocol::{
    Span,
    ast::{Expr, ExternalArgument, Traverse},
};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const DEFAULT_LINE_COUNT: usize = 10;
const DEFAULT_FOLLOW_LINE_COUNT: usize = 20;

struct TailFixData<'a> {
    count: Option<usize>,
    filename: Option<&'a str>,
    follow: bool,
    unsupported: bool,
    expr_span: Span,
}

struct TailArgument<'a> {
    text: &'a str,
    span: Span,
}

fn parse_arguments<'a>(arguments: &[TailArgument<'a>], expr_span: Span) -> TailFixData<'a> {
    let mut parsed = TailFixData {
        count: None,
        filename: None,
        follow: false,
        unsupported: false,
        expr_span,
    };

    let mut index = 0;
    while let Some(argument) = arguments.get(index) {
        let text = argument.text;
        match text {
            "-f" | "-F" | "--follow" => parsed.follow = true,
            "-n" | "--lines" => {
                index += 1;
                parsed.take_count(arguments.get(index).map(|next| next.text));
            }
            _ => {
                if let Some(value) = text.strip_prefix("--lines=") {
                    parsed.take_count(Some(value));
                } else if let Some(value) = text.strip_prefix("-n") {
                    parsed.take_count(Some(value));
                } else if let Some(value) = text.strip_prefix('-') {
                    if value.chars().all(char::is_numeric) && !value.is_empty() {
                        parsed.take_count(Some(value));
                    }
                } else if parsed.filename.is_none() {
                    parsed.filename = Some(text);
                }
            }
        }
        index += 1;
    }

    parsed
}

impl<'a> TailFixData<'a> {
    fn take_count(&mut self, value: Option<&'a str>) {
        let line_count = value
            .filter(|text| text.chars().all(char::is_numeric))
            .and_then(|text| text.parse::<usize>().ok());

        match line_count {
            Some(count) => self.count = Some(count),
            None => self.unsupported = true,
        }
    }

    fn line_count(&self, default: usize) -> usize {
        self.count.unwrap_or(default)
    }
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

                let arguments: Vec<TailArgument> =
                    args.iter()
                        .map(|arg| {
                            let span = match arg {
                                ExternalArgument::Regular(expr)
                                | ExternalArgument::Spread(expr) => expr.span,
                            };
                            TailArgument {
                                text: context.span_text(span),
                                span,
                            }
                        })
                        .collect();

                let fix_data = parse_arguments(&arguments, expr.span);

                let detection = arguments.iter().fold(
                    Detection::from_global_span("Use 'last N' to get the last N items", head.span)
                        .with_primary_label("external 'tail'"),
                    |detection, argument| {
                        if argument.text.starts_with('-') {
                            detection.with_extra_label("line count", argument.span)
                        } else {
                            detection.with_extra_label("file", argument.span)
                        }
                    },
                );

                vec![(detection, fix_data)]
            },
            &mut results,
        );

        results
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        if fix_data.unsupported {
            return None;
        }

        if fix_data.follow {
            let file = fix_data.filename?;
            let count = fix_data.line_count(DEFAULT_FOLLOW_LINE_COUNT);

            return Some(Fix {
                explanation: "Use 'watch' to monitor file changes. Nu's watch executes a closure \
                              when the file changes, similar to 'tail -f'. Note: this is \
                              event-based, not continuous streaming."
                    .into(),
                replacements: vec![Replacement::new(
                    fix_data.expr_span,
                    format!("watch {file} {{ open --raw {file} | lines | last {count} }}"),
                )],
            });
        }

        let count = fix_data.line_count(DEFAULT_LINE_COUNT);
        let replacement = fix_data.filename.map_or_else(
            || format!("last {count}"),
            |file| format!("open --raw {file} | lines | last {count}"),
        );

        Some(Fix {
            explanation: "Use 'last' with cleaner syntax: 'last N' instead of 'tail -N'".into(),
            replacements: vec![Replacement::new(fix_data.expr_span, replacement)],
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
