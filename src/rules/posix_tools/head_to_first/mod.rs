use nu_protocol::{
    Span,
    ast::{Expr, ExternalArgument, Pipeline},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct HeadFixData {
    count: Option<isize>,
    filename_span: Option<Span>,
    combined_span: Span,
}

struct UseBuiltinHead;

fn is_lines_arg(a: &ExternalArgument, ctx: &LintContext) -> bool {
    let t = ctx.expr_text(a.expr());
    t == "--lines" || t == "-n"
}

fn is_file_arg(a: &ExternalArgument, ctx: &LintContext) -> bool {
    let t = ctx.expr_text(a.expr());
    t.chars().next().is_some_and(|f| f != '-')
}

fn is_piped_to_lines(pipeline: &Pipeline, idx: usize, context: &LintContext) -> bool {
    pipeline.elements.get(idx + 1).is_some_and(|next| {
        matches!(&next.expr.expr, Expr::Call(call) if call.is_call_to_command("lines", context))
    })
}

fn extract_count(args: &[ExternalArgument], context: &LintContext) -> Option<isize> {
    let count_idx = args.iter().position(|e| is_lines_arg(e, context))?;
    if count_idx + 1 >= args.len() {
        return None;
    }
    context
        .expr_text(args[count_idx + 1].expr())
        .parse::<isize>()
        .ok()
}

fn extract_filename_span(args: &[ExternalArgument], context: &LintContext) -> Option<Span> {
    args.iter()
        .zip(args.iter().skip(1))
        .find(|(prev, curr)| !is_lines_arg(prev, context) && is_file_arg(curr, context))
        .map(|(_, file_arg)| file_arg.expr().span)
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, HeadFixData)> {
    pipeline
        .elements
        .iter()
        .enumerate()
        .filter_map(|(idx, element)| {
            let Expr::ExternalCall(head, args) = &element.expr.expr else {
                return None;
            };

            let cmd_text = context.expr_text(head);
            if cmd_text != "head" {
                return None;
            }

            // Must be followed by `lines`
            if !is_piped_to_lines(pipeline, idx, context) {
                return None;
            }

            let count = extract_count(args, context);
            let filename_span = extract_filename_span(args, context);

            // Compute combined span from head to lines element
            let lines_span = pipeline.elements[idx + 1].expr.span;
            let combined_span = Span::new(element.expr.span.start, lines_span.end);

            let fix_data = HeadFixData {
                count,
                filename_span,
                combined_span,
            };

            let detection =
                Detection::from_global_span("Use 'first N' to get the first N items", head.span)
                    .with_primary_label("external 'head'");

            Some((detection, fix_data))
        })
        .collect()
}

impl DetectFix for UseBuiltinHead {
    type FixInput<'a> = HeadFixData;

    fn id(&self) -> &'static str {
        "head_to_first"
    }

    fn short_description(&self) -> &'static str {
        "`head` replaceable with `first`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/first.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let count = fix_data.count.unwrap_or(10);

        let replacement = fix_data.filename_span.map_or_else(
            || format!("lines | first {count}"),
            |span| {
                let filename = context.span_text(span);
                format!("open {filename} | lines | first {count}")
            },
        );

        Some(Fix {
            explanation: "Use 'first' with cleaner syntax".into(),
            replacements: vec![Replacement {
                span: fix_data.combined_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinHead;

#[cfg(test)]
mod tests;
