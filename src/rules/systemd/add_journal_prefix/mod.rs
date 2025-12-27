//! Rule: `add_journal_prefix`
//!
//! Adds systemd journal log level prefixes to print/echo statements.

use nu_protocol::ast::{Block, Expr, Expression, Traverse};

use super::{
    LogLevel, PrefixStatus, extract_first_string_part, is_print_or_echo, pipeline_contains_print,
};
use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt, string::StringFormat},
    context::LintContext,
    rule::{DetectFix, Rule},
    rules::systemd::strip_keyword_prefix,
    violation::{Detection, Fix, Replacement},
};

/// Semantic fix data: stores the string format and detected log level
pub struct FixData {
    /// String format of the argument
    string_format: StringFormat,
    /// Span of the argument to replace
    arg_span: nu_protocol::Span,
    /// Detected log level from message content
    level: LogLevel,
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
    let string_format = StringFormat::from_expression(arg_expr, ctx)?;
    let message_content = extract_first_string_part(arg_expr, ctx)?;

    match PrefixStatus::check(&message_content) {
        PrefixStatus::Missing => {
            let level = LogLevel::detect_from_message(&message_content);
            let detected = Detection::from_global_span("Missing systemd journal prefix", expr.span)
                .with_primary_label("print/echo without journal prefix")
                .with_help(format!(
                    "Add <{}> prefix for systemd journal logging",
                    level.numeric_str()
                ));

            let fix_data = FixData {
                string_format,
                arg_span: arg_expr.span,
                level,
            };

            Some((detected, fix_data))
        }
        PrefixStatus::Valid => None,
    }
}

fn check_block(block: &Block, ctx: &LintContext) -> Vec<(Detection, FixData)> {
    let mut results = Vec::new();

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

        if let Some(result) = check_print_or_echo_call(&first_element.expr, ctx) {
            results.push(result);
        }
    }

    results
}

struct AddJournalPrefix;

impl DetectFix for AddJournalPrefix {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "add_journal_prefix"
    }

    fn explanation(&self) -> &'static str {
        "Add systemd journal log level prefixes to print/echo statements."
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut results = check_block(context.ast, context);

        context.ast.flat_map(
            context.working_set,
            &|expr| {
                if let Some(block_id) = expr.extract_block_id() {
                    let block = context.working_set.get_block(block_id);
                    return check_block(block, context);
                }
                vec![]
            },
            &mut results,
        );

        results
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let original_content = fix_data.string_format.content();
        let cleaned = strip_keyword_prefix(original_content);
        let new_content = format!("<{}>{cleaned}", fix_data.level.numeric_str());
        let fixed = fix_data.string_format.reconstruct(&new_content);

        Some(Fix::with_explanation(
            format!("Add <{}> prefix", fix_data.level.numeric_str()),
            vec![Replacement::new(fix_data.arg_span, fixed)],
        ))
    }
}

pub static RULE: &dyn Rule = &AddJournalPrefix;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
