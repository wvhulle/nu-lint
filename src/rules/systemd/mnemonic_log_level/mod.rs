//! Rule: `prefer_keyword_journal_prefix`
//!
//! Replaces numeric systemd journal prefixes with keyword prefixes.

use nu_protocol::ast::{Block, Expr, Expression, Traverse};

use super::{
    FixGenerator, LogLevel, PrefixStatus, extract_first_string_part, is_print_or_echo,
    pipeline_contains_print,
};
use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

fn create_violation(
    span: nu_protocol::Span,
    level: LogLevel,
    arg_expr: &Expression,
    ctx: &LintContext,
) -> Violation {
    let fix_gen = FixGenerator::new(level, arg_expr, ctx);
    let fixed_string = fix_gen.replace_numeric_prefix(level.numeric_str());

    Violation::new("Numeric journal prefix", span).with_fix(Fix::with_explanation(
        format!(
            "Replace <{}> with <{}>",
            level.numeric_str(),
            level.keyword()
        ),
        vec![Replacement::new(arg_expr.span, fixed_string)],
    ))
}

fn check_print_or_echo_call(expr: &Expression, ctx: &LintContext) -> Option<Violation> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    let command_name = call.get_call_name(ctx);
    if !matches!(command_name.as_str(), "print" | "echo") {
        return None;
    }

    let arg_expr = call.get_first_positional_arg()?;
    let message_content = extract_first_string_part(arg_expr, ctx)?;

    match PrefixStatus::check(&message_content) {
        PrefixStatus::Numeric(level) => Some(create_violation(expr.span, level, arg_expr, ctx)),
        PrefixStatus::Missing | PrefixStatus::Valid => None,
    }
}

fn check_block(block: &Block, ctx: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (i, pipeline) in block.pipelines.iter().enumerate() {
        let Some(first_element) = pipeline.elements.first() else {
            continue;
        };

        if !is_print_or_echo(&first_element.expr, ctx) {
            continue;
        }

        // Skip if adjacent pipeline also contains print (consecutive prints = UI
        // output)
        let prev_has_print = i > 0
            && block
                .pipelines
                .get(i - 1)
                .is_some_and(|p| pipeline_contains_print(p, ctx));

        let next_has_print = block
            .pipelines
            .get(i + 1)
            .is_some_and(|p| pipeline_contains_print(p, ctx));

        if prev_has_print || next_has_print {
            continue;
        }

        if let Some(v) = check_print_or_echo_call(&first_element.expr, ctx) {
            violations.push(v);
        }
    }

    violations
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = check_block(context.ast, context);

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Some(block_id) = expr.extract_block_id() {
                let block = context.working_set.get_block(block_id);
                return check_block(block, context);
            }
            vec![]
        },
        &mut violations,
    );

    violations
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_keyword_journal_prefix",
        "Use mnemonic log levels instead of numeric ones for systemd journal log levels.",
        check,
        LintLevel::Hint,
    )
    .with_doc_url(
        "https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html#SyslogLevelPrefix=",
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
