use std::collections::HashMap;

use nu_protocol::{
    Span, VarId,
    ast::{Expr, Expression, FindMapResult, PathMember},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

fn cell_path_has_member(members: &[PathMember], member_name: &str) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            PathMember::String { val, .. } if val == member_name
        )
    })
}

fn is_exit_code_access(expr: &Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    expr.find_map(context.working_set, &|inner_expr| {
        if let Expr::FullCellPath(cell_path) = &inner_expr.expr
            && cell_path_has_member(&cell_path.tail, "exit_code")
        {
            log::debug!("Found .exit_code field access");
            return FindMapResult::Found(());
        }

        if let Expr::Call(call) = &inner_expr.expr
            && call.is_call_to_command("get", context)
            && call.get_positional_arg(0).is_some_and(|arg| {
                matches!(&arg.expr,
                    Expr::CellPath(cp) if cell_path_has_member(&cp.members, "exit_code")
                ) || matches!(&arg.expr, Expr::String(s) if s == "exit_code")
            })
        {
            log::debug!("Found 'get exit_code' command");
            return FindMapResult::Found(());
        }

        FindMapResult::Continue
    })
    .is_some()
}

fn has_complete_call(expr: &Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    expr.find_map(context.working_set, &|inner_expr| {
        if let Expr::Call(inner_call) = &inner_expr.expr
            && inner_call.is_call_to_command("complete", context)
        {
            FindMapResult::Found(())
        } else {
            FindMapResult::Continue
        }
    })
    .is_some()
}

fn extract_complete_assignment(
    expr: &Expression,
    context: &LintContext,
) -> Option<(VarId, String, Span, bool, Option<String>)> {
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

    let exit_code_checked = is_exit_code_access(value_arg, context);
    let command_name = value_arg.extract_external_command_name(context);

    Some((var_id, var_name, expr.span, exit_code_checked, command_name))
}

fn find_complete_assignments(
    context: &LintContext,
) -> HashMap<VarId, (String, Span, bool, Option<String>)> {
    use nu_protocol::ast::Traverse;

    let mut assignments = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            extract_complete_assignment(expr, context)
                .into_iter()
                .collect()
        },
        &mut assignments,
    );

    assignments
        .into_iter()
        .map(|(id, name, span, checked, cmd)| (id, (name, span, checked, cmd)))
        .collect()
}

fn find_exit_code_checks(context: &LintContext) -> HashMap<VarId, Span> {
    use nu_protocol::ast::Traverse;

    let mut checks = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| expr.extract_field_access("exit_code").into_iter().collect(),
        &mut checks,
    );

    checks.into_iter().collect()
}

fn check(context: &LintContext) -> Vec<Violation> {
    let assignments = find_complete_assignments(context);
    let checks = find_exit_code_checks(context);

    assignments
        .iter()
        .filter(|(var_id, (_, _, checked_inline, _))| {
            if *checked_inline {
                log::debug!("Skipping variable {var_id:?} - exit_code checked in assignment");
                return false;
            }
            if checks.contains_key(var_id) {
                log::debug!(
                    "Skipping variable {var_id:?} - exit_code checked via variable reference"
                );
                return false;
            }
            true
        })
        .map(|(_, (var_name, span, _, cmd_name))| {
            let cmd_desc = cmd_name
                .as_ref()
                .map_or(String::new(), |c| format!("'{c}' "));

            Violation::new_dynamic(
                "check_complete_exit_code",
                format!(
                    "External command {cmd_desc}result '{var_name}' stored but exit code not \
                     checked"
                ),
                *span,
            )
            .with_suggestion_dynamic(format!(
                "Check the exit code to handle command failures. For example:\nif \
                 ${var_name}.exit_code != 0 {{\n\x20   error make {{msg: \
                 '{cmd_desc}failed'}}\n}}\nOr use inline checking:\nlet success = ({cmd_desc}| \
                 complete | get exit_code) == 0"
            ))
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "check_complete_exit_code",
        "Check exit codes when using 'complete' to capture external command results",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
