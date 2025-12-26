//! Rule: `prefer_keyword_journal_prefix`
//!
//! Replaces numeric systemd journal prefixes with keyword prefixes.

use nu_protocol::{
    Span,
    ast::{Block, Expr, Expression, Traverse},
};

use super::{
    LogLevel, PrefixStatus, extract_first_string_part, is_print_or_echo, pipeline_contains_print,
};
use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
pub struct FixData {
    arg_span: Span,
    level: LogLevel,
}

fn detect_violation(
    violation_span: nu_protocol::Span,
    level: LogLevel,
    arg_expr: &Expression,
) -> (Detection, FixData) {
    let violation = Detection::from_global_span("Numeric journal prefix", violation_span);
    let fix_data = FixData {
        arg_span: arg_expr.span,
        level,
    };
    (violation, fix_data)
}

fn check_print_or_echo_call(expr: &Expression, ctx: &LintContext) -> Option<(Detection, FixData)> {
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
        PrefixStatus::Numeric(level) => Some(detect_violation(expr.span, level, arg_expr)),
        PrefixStatus::Missing | PrefixStatus::Valid => None,
    }
}

fn check_block(block: &Block, ctx: &LintContext) -> Vec<(Detection, FixData)> {
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

struct AttachLoglevelToLogStatement;

impl DetectFix for AttachLoglevelToLogStatement {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "attach_loglevel_to_log_statement"
    }

    fn explanation(&self) -> &'static str {
        "Use mnemonic log levels instead of numeric ones for systemd journal log levels."
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
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

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // Get the original argument text and replace the numeric prefix with keyword
        let arg_text = ctx.get_span_text(fix_data.arg_span);
        let fixed_string = arg_text.replacen(
            &format!("<{}>", fix_data.level.numeric_str()),
            &format!("<{}>", fix_data.level.keyword()),
            1,
        );

        Some(Fix::with_explanation(
            format!(
                "Replace <{}> with <{}>",
                fix_data.level.numeric_str(),
                fix_data.level.keyword()
            ),
            vec![Replacement::new(fix_data.arg_span, fixed_string)],
        ))
    }
}

pub static RULE: &dyn Rule = &AttachLoglevelToLogStatement;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
