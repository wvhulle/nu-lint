use std::collections::HashMap;

use nu_protocol::{
    Span, VarId,
    ast::{Expr, Expression, FindMapResult, PathMember},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt, span::SpanExt},
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
        _ => cmd_expr.span.source_code(context).to_string(),
    }
}

fn has_dangerous_external_command(expr: &Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    expr.find_map(context.working_set, &|inner| {
        let Expr::ExternalCall(cmd_expr, args) = &inner.expr else {
            return FindMapResult::Continue;
        };

        let cmd_name = extract_command_name(cmd_expr, context);
        let is_dangerous = has_external_side_effect(
            &cmd_name,
            ExternEffect::CommonEffect(CommonEffect::Dangerous),
            context,
            args,
        );

        if is_dangerous {
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

    matches!(call.get_call_name(context).as_str(), "let" | "mut").then_some(())?;

    let (var_id, var_name, _) = call.extract_variable_declaration(context)?;
    let value_arg = call.get_positional_arg(1)?;

    has_complete_call(value_arg, context).then_some(())?;

    if !has_dangerous_external_command(value_arg, context) {
        log::debug!("Skipping '{var_name}' - external command is not marked as dangerous");
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

    assignments.into_iter().map(|a| (a.var_id, a)).collect()
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

fn is_exit_code_unchecked(
    assignment: &CompleteAssignment,
    exit_code_checks: &HashMap<VarId, Span>,
) -> bool {
    if assignment.exit_code_checked_inline {
        log::debug!(
            "Skipping variable {:?} - exit_code checked in assignment",
            assignment.var_id
        );
        return false;
    }

    if exit_code_checks.contains_key(&assignment.var_id) {
        log::debug!(
            "Skipping variable {:?} - exit_code checked via variable reference",
            assignment.var_id
        );
        return false;
    }

    true
}

fn create_violation(assignment: &CompleteAssignment) -> Detection {
    let cmd_desc = assignment
        .command_name
        .as_ref()
        .map_or(String::new(), |c| format!("'{c}' "));

    Detection::from_global_span(
        format!(
            "External command {cmd_desc}result '{}' stored but exit code not checked",
            assignment.var_name
        ),
        assignment.span,
    )
    .with_primary_label("without exit_code check")
    .with_help(format!(
        "Check the exit code to handle command failures. For example:\nif ${}.exit_code != 0 \
         {{\n\x20   error make {{msg: '{cmd_desc}failed'}}\n}}\nOr use inline checking:\nlet \
         success = ({cmd_desc}| complete | get exit_code) == 0",
        assignment.var_name
    ))
}

struct CheckCompleteExitCode;

impl DetectFix for CheckCompleteExitCode {
    type FixInput = ();

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

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
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
