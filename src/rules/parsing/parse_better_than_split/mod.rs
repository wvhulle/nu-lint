use nu_protocol::ast::{Argument, Expr, Expression};

use super::is_split_call;
use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn contains_split_in_expression(expr: &Expression, ctx: &LintContext) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            is_split_call(call, ctx)
                || call.arguments.iter().any(|arg| {
                    matches!(arg,
                        Argument::Positional(e) | Argument::Named((_, _, Some(e)))
                        if contains_split_in_expression(e, ctx)
                    )
                })
        }
        Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => ctx
            .working_set
            .get_block(*id)
            .pipelines
            .iter()
            .flat_map(|p| &p.elements)
            .any(|elem| contains_split_in_expression(&elem.expr, ctx)),
        Expr::FullCellPath(path) => contains_split_in_expression(&path.head, ctx),
        Expr::BinaryOp(left, _, right) => {
            contains_split_in_expression(left, ctx) || contains_split_in_expression(right, ctx)
        }
        Expr::UnaryNot(inner) => contains_split_in_expression(inner, ctx),
        _ => false,
    }
}

fn check_each_with_split(expr: &Expression, ctx: &LintContext) -> Option<Detection> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };
    if !call.is_call_to_command("each", ctx) {
        return None;
    }

    let has_split = call
        .arguments
        .iter()
        .any(|arg| matches!(arg, Argument::Positional(e) if contains_split_in_expression(e, ctx)));

    has_split.then(|| {
        Detection::from_global_span(
            "Manual splitting with 'each' and 'split row' - consider using 'parse'",
            call.span(),
        )
        .with_primary_label("manual split pattern")
        .with_help(
            "Use 'parse \"{field0} {field1}\"' for structured text extraction instead of 'each' \
             with 'split row'. For complex delimiters, use 'parse --regex' with named capture \
             groups like '(?P<field0>.*)delimiter(?P<field1>.*)'",
        )
    })
}

struct EachSplitRule;

impl DetectFix for EachSplitRule {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "parse_better_than_split"
    }

    fn explanation(&self) -> &'static str {
        "The 'parse' command is often better than the 'split row' pattern."
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/parse.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let violations =
            context.detect(|expr, ctx| check_each_with_split(expr, ctx).into_iter().collect());
        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &EachSplitRule;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
