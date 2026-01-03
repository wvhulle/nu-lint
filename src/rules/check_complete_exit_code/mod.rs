use std::collections::HashMap;

use nu_protocol::{
    Span, VarId,
    ast::{Expr, Expression, FindMapResult, PathMember},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    effect::{
        CommonEffect,
        external::{ExternEffect, has_external_side_effect},
    },
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct CompleteAssignment {
    var_id: VarId,
    var_name: String,
    span: Span,
    exit_code_checked_inline: bool,
    command_name: Option<String>,
}

fn cell_path_has_member(members: &[PathMember], member_name: &str) -> bool {
    members
        .iter()
        .any(|m| matches!(m, PathMember::String { val, .. } if val == member_name))
}

fn is_exit_code_cell_path(expr: &Expr) -> bool {
    matches!(expr, Expr::FullCellPath(cp) if cell_path_has_member(&cp.tail, "exit_code"))
}

fn is_get_exit_code_call(expr: &Expr, context: &LintContext) -> bool {
    let Expr::Call(call) = expr else { return false };

    call.is_call_to_command("get", context) && call.get_positional_arg(0).is_some_and(|arg| {
        matches!(&arg.expr, Expr::CellPath(cp) if cell_path_has_member(&cp.members, "exit_code"))
            || matches!(&arg.expr, Expr::String(s) if s == "exit_code")
    })
}

fn has_exit_code_access(expr: &Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    expr.find_map(context.working_set, &|inner| {
        if is_exit_code_cell_path(&inner.expr) || is_get_exit_code_call(&inner.expr, context) {
            log::debug!("Found exit_code access");
            FindMapResult::Found(())
        } else {
            FindMapResult::Continue
        }
    })
    .is_some()
}

fn has_complete_call(expr: &Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    expr.find_map(context.working_set, &|inner| {
        if matches!(&inner.expr, Expr::Call(call) if call.is_call_to_command("complete", context)) {
            FindMapResult::Found(())
        } else {
            FindMapResult::Continue
        }
    })
    .is_some()
}

fn extract_command_name(cmd_expr: &Expression, context: &LintContext) -> String {
    match &cmd_expr.expr {
        Expr::String(s) => s.clone(),
        Expr::GlobPattern(pattern, _) => pattern.clone(),
        _ => context.plain_text(cmd_expr.span).to_string(),
    }
}

fn has_external_command_with_likely_errors(expr: &Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    expr.find_map(context.working_set, &|inner| {
        let Expr::ExternalCall(cmd_expr, args) = &inner.expr else {
            return FindMapResult::Continue;
        };

        let cmd_name = extract_command_name(cmd_expr, context);
        if has_external_side_effect(
            &cmd_name,
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            context,
            args,
        ) {
            FindMapResult::Found(())
        } else {
            FindMapResult::Continue
        }
    })
    .is_some()
}

fn try_extract_complete_assignment(
    expr: &Expression,
    context: &LintContext,
) -> Option<CompleteAssignment> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if !matches!(call.get_call_name(context).as_str(), "let" | "mut") {
        return None;
    }

    let (var_id, var_name, _) = call.extract_variable_declaration(context)?;
    let value_arg = call.get_positional_arg(1)?;

    if !has_complete_call(value_arg, context) {
        return None;
    }

    if !has_external_command_with_likely_errors(value_arg, context) {
        log::debug!("Skipping '{var_name}' - external command not marked as likely to error");
        return None;
    }

    Some(CompleteAssignment {
        var_id,
        var_name,
        span: expr.span,
        exit_code_checked_inline: has_exit_code_access(value_arg, context),
        command_name: value_arg.extract_external_command_name(context),
    })
}

fn collect_complete_assignments(context: &LintContext) -> HashMap<VarId, CompleteAssignment> {
    use nu_protocol::ast::Traverse;

    let mut assignments = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            try_extract_complete_assignment(expr, context)
                .into_iter()
                .collect()
        },
        &mut assignments,
    );

    assignments
        .into_iter()
        .map(|assignment| (assignment.var_id, assignment))
        .collect()
}

fn collect_exit_code_checks(context: &LintContext) -> HashMap<VarId, Span> {
    use nu_protocol::ast::Traverse;

    let mut checks = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| expr.extract_field_access("exit_code").into_iter().collect(),
        &mut checks,
    );

    checks.into_iter().collect()
}

fn is_exit_code_checked(
    assignment: &CompleteAssignment,
    exit_code_checks: &HashMap<VarId, Span>,
) -> bool {
    assignment.exit_code_checked_inline || exit_code_checks.contains_key(&assignment.var_id)
}

fn is_exit_code_unchecked(
    assignment: &CompleteAssignment,
    exit_code_checks: &HashMap<VarId, Span>,
) -> bool {
    let checked = is_exit_code_checked(assignment, exit_code_checks);

    if checked {
        let reason = if assignment.exit_code_checked_inline {
            "exit_code checked in assignment"
        } else {
            "exit_code checked via variable reference"
        };
        log::debug!("Skipping variable {:?} - {reason}", assignment.var_id);
    }

    !checked
}

fn create_violation(assignment: &CompleteAssignment) -> Detection {
    let cmd_desc = assignment
        .command_name
        .as_deref()
        .map_or_else(String::new, |cmd| format!("'{cmd}' "));

    let message = format!(
        "External command {cmd_desc}result '{}' stored but exit code not checked",
        assignment.var_name
    );

    let help_message = format!(
        "Check the exit code to handle command failures. For example:\nif ${}.exit_code != 0 \
         {{\n\x20   error make {{msg: '{cmd_desc}failed'}}\n}}\nOr use inline checking:\nlet \
         success = ({cmd_desc}| complete | get exit_code) == 0",
        assignment.var_name
    );

    Detection::from_global_span(message, assignment.span)
        .with_primary_label("without exit_code check")
        .with_help(help_message)
}

struct CheckCompleteExitCode;

impl DetectFix for CheckCompleteExitCode {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "check_complete_exit_code"
    }

    fn explanation(&self) -> &'static str {
        "Check exit codes when using 'complete' to capture dangerous external command results"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/complete.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let assignments = collect_complete_assignments(context);
        let exit_code_checks = collect_exit_code_checks(context);

        Self::no_fix(
            assignments
                .values()
                .filter(|a| is_exit_code_unchecked(a, &exit_code_checks))
                .map(create_violation)
                .collect(),
        )
    }
}

pub static RULE: &dyn Rule = &CheckCompleteExitCode;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
