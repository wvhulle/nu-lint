use nu_protocol::{
    ENV_VARIABLE_ID, Span,
    ast::{Block, Expr, Expression, PathMember, Pipeline, Traverse},
};

use crate::{
    LintLevel,
    ast::expression::ExpressionExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

#[derive(Clone)]
struct FixData {
    external_cmd_span: Span,
    last_exit_code_span: Span,
    external_cmd_text: String,
}

/// Check if expression is exactly `$env.LAST_EXIT_CODE`
fn is_last_exit_code_access(expr: &Expression) -> bool {
    let Expr::FullCellPath(cell_path) = &expr.expr else {
        return false;
    };

    // Check if head is $env
    let is_env_var =
        matches!(&cell_path.head.expr, Expr::Var(var_id) if *var_id == ENV_VARIABLE_ID);

    // Check if tail is exactly one member: "LAST_EXIT_CODE"
    let is_exact_member = cell_path.tail.len() == 1
        && matches!(&cell_path.tail[0], PathMember::String { val, .. } if val == "LAST_EXIT_CODE");

    is_env_var && is_exact_member
}

/// Find the first external command in a pipeline
fn find_external_command(pipeline: &Pipeline, context: &LintContext) -> Option<(Span, String)> {
    pipeline.elements.iter().find_map(|element| {
        matches!(&element.expr.expr, Expr::ExternalCall(_, _)).then(|| {
            (
                element.expr.span,
                context.plain_text(element.expr.span).to_string(),
            )
        })
    })
}

/// Find `LAST_EXIT_CODE` access anywhere in a pipeline
fn find_last_exit_code_check(pipeline: &Pipeline, context: &LintContext) -> Option<Span> {
    use nu_protocol::ast::FindMapResult;

    pipeline.elements.iter().find_map(|element| {
        element.expr.find_map(context.working_set, &|expr| {
            if is_last_exit_code_access(expr) {
                FindMapResult::Found(expr.span)
            } else {
                FindMapResult::Continue
            }
        })
    })
}

/// Check for the pattern: `external_command` followed by `LAST_EXIT_CODE` check
fn check_pipeline_pairs(block: &Block, context: &LintContext) -> Vec<(Detection, Option<FixData>)> {
    block
        .pipelines
        .iter()
        .enumerate()
        .filter_map(|(idx, pipeline)| {
            // Find external command in current pipeline
            let (cmd_span, cmd_text) = find_external_command(pipeline, context)?;

            // Check next pipeline for LAST_EXIT_CODE access
            let next_pipeline = block.pipelines.get(idx + 1)?;
            let exit_code_span = find_last_exit_code_check(next_pipeline, context)?;

            // Build fix data and detection
            let fix_data = FixData {
                external_cmd_span: cmd_span,
                last_exit_code_span: exit_code_span,
                external_cmd_text: cmd_text,
            };

            let detection = Detection::from_global_span(
                "$env.LAST_EXIT_CODE is fragile and can be overwritten by hooks. Use 'complete' \
                 with inline exit_code check instead.",
                exit_code_span,
            )
            .with_primary_label("fragile exit code check");

            Some((detection, Some(fix_data)))
        })
        .collect()
}

/// Detect violations across all blocks (main and nested)
fn detect_in_all_blocks(context: &LintContext) -> Vec<(Detection, Option<FixData>)> {
    let mut all_violations = check_pipeline_pairs(context.ast, context);

    // Recursively check all nested blocks (functions, closures, etc.)
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            expr.extract_block_id()
                .map(|block_id| {
                    let block = context.working_set.get_block(block_id);
                    check_pipeline_pairs(block, context)
                })
                .unwrap_or_default()
        },
        &mut all_violations,
    );

    all_violations
}

struct AvoidLastExitCode;

impl DetectFix for AvoidLastExitCode {
    type FixInput<'a> = Option<FixData>;

    fn id(&self) -> &'static str {
        "avoid_last_exit_code"
    }

    fn short_description(&self) -> &'static str {
        "Avoid fragile $env.LAST_EXIT_CODE checks; use 'complete' with inline exit_code checks"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "$env.LAST_EXIT_CODE is fragile because it can be overwritten by any subsequent \
             external command or shell hook before you check it. Instead, pipe the command to \
             'complete' and check the exit_code field inline: (cmd | complete).exit_code != 0. \
             This pattern tightly couples the exit code to its specific command, making it \
             impossible for hooks or other commands to interfere.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/complete.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Error)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        detect_in_all_blocks(context)
    }

    fn fix(&self, _context: &LintContext, data: &Self::FixInput<'_>) -> Option<Fix> {
        let fix_data = data.as_ref()?;

        let inline_complete = format!("({} | complete).exit_code", fix_data.external_cmd_text);

        Some(Fix::with_explanation(
            "Replace fragile $env.LAST_EXIT_CODE check with inline complete pattern".to_string(),
            vec![
                Replacement::new(fix_data.external_cmd_span, String::new()),
                Replacement::new(fix_data.last_exit_code_span, inline_complete),
            ],
        ))
    }
}

pub static RULE: &dyn Rule = &AvoidLastExitCode;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
