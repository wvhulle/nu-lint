use nu_protocol::ast::{Expr, ExternalArgument, Traverse};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct HeadFixData<'a> {
    count: Option<isize>,
    filename: Option<&'a str>,
    expr_span: nu_protocol::Span,
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

impl DetectFix for UseBuiltinHead {
    type FixInput<'a> = HeadFixData<'a>;

    fn id(&self) -> &'static str {
        "head_to_first"
    }

    fn short_description(&self) -> &'static str {
        "Use Nu's 'first' command instead of 'head' for cleaner syntax"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/first.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut results = Vec::new();

        context.ast.flat_map(
            context.working_set,
            &|expr| {
                let Expr::ExternalCall(head, args) = &expr.expr else {
                    return vec![];
                };

                let cmd_text = context.span_text(head.span);
                if cmd_text != "head" || args.len() <= 2 {
                    return vec![];
                }

                let mut fix_data = HeadFixData {
                    expr_span: expr.span,
                    count: None,
                    filename: None,
                };

                let count_it = args.iter().position(|e| is_lines_arg(e, context));

                if let Some(count_it) = count_it
                    && count_it < args.len()
                    && let Ok(count) = context
                        .expr_text(args[count_it + 1].expr())
                        .parse::<isize>()
                {
                    fix_data.count = Some(count);
                }

                if let Some((_, file_arg)) = args
                    .iter()
                    .zip(args.iter().skip(1))
                    .find(|(i, n)| !is_lines_arg(i, context) && is_file_arg(n, context))
                {
                    fix_data.filename = Some(context.expr_text(file_arg.expr()));
                } else {
                    return vec![];
                }

                let detection = Detection::from_global_span(
                    "Use 'first N' to get the first N items",
                    head.span,
                )
                .with_primary_label("external 'head'");

                vec![(detection, fix_data)]
            },
            &mut results,
        );

        results
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let count = fix_data.count.unwrap_or(10);

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
