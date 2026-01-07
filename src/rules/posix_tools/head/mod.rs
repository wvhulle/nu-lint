use nu_protocol::ast::{Expr, Traverse};

use crate::{
    LintLevel,
    ast::expression::ExpressionExt,
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

                let mut fix_data = HeadFixData {
                    expr_span: expr.span,
                    count: None,
                    filename: None,
                };

                let mut count_it = args.iter().peekable();
                while let Some(el) = count_it.next() {
                    let current = context.plain_text(el.expr().span);
                    if current == "-n"
                        && let Some(number) = count_it.peek()
                    {
                        let number = number.expr().span_text(context).parse::<isize>().unwrap();
                        fix_data.count = Some(number);
                    }
                }

                let mut file_name_it = args.iter().rev().peekable();
                while let Some(el) = file_name_it.next() {
                    let current = context.plain_text(el.expr().span);
                    if current != "-n"
                        && let Some(flag) = file_name_it.peek()
                        && flag.expr().span_text(context) != "-n"
                    {
                        fix_data.filename = Some(current);
                    }
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
