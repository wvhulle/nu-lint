use std::collections::{HashMap, HashSet};

use lsp_types::DiagnosticTag;
use nu_protocol::{
    Span, VarId,
    ast::{Argument, Call, Expr, Expression, Operator, Traverse},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

type EmptyListVar = (VarId, String, Span);
type DirectCopyVars = Vec<VarId>;
type AnalysisPattern = (Vec<EmptyListVar>, DirectCopyVars);

fn is_literal_list(expr: &Expression) -> bool {
    match &expr.expr {
        Expr::List(_) => true,
        Expr::FullCellPath(cell_path) => matches!(&cell_path.head.expr, Expr::List(_)),
        Expr::Keyword(keyword) => is_literal_list(&keyword.expr),
        _ => false,
    }
}

fn matches_transformation_pattern(
    expr: &Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Call(call) => call.is_call_to_command("if", context),
        Expr::BinaryOp(_lhs, op, rhs) => {
            matches!(op.expr, Expr::Operator(Operator::Assignment(_)))
                && has_transformation_in_append(rhs, context, loop_var_name)
        }
        _ => false,
    }
}

fn has_transformation_or_filter(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);
    block
        .all_elements()
        .iter()
        .any(|elem| matches_transformation_pattern(&elem.expr, context, loop_var_name))
}

fn has_transformation_in_append(
    expr: &Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    match &expr.expr {
        Expr::Call(call) if call.is_call_to_command("append", context) => {
            let Some(arg) = call.arguments.first() else {
                return false;
            };

            let (Argument::Positional(arg_expr) | Argument::Unknown(arg_expr)) = arg else {
                return false;
            };

            match &arg_expr.expr {
                Expr::Var(_) => {
                    let var_name = arg_expr.span_text(context);
                    var_name != loop_var_name
                }
                Expr::FullCellPath(cell_path) if matches!(&cell_path.head.expr, Expr::Var(_)) => {
                    let var_name = cell_path.head.span_text(context);
                    var_name == loop_var_name && !cell_path.tail.is_empty()
                }
                _ => true,
            }
        }
        Expr::FullCellPath(cell_path) => {
            has_transformation_in_append(&cell_path.head, context, loop_var_name)
        }
        Expr::Block(block_id) | Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block
                .all_elements()
                .iter()
                .any(|elem| has_transformation_in_append(&elem.expr, context, loop_var_name))
        }
        _ => false,
    }
}

type EmptyListVarsMap = HashMap<VarId, (String, Span)>;

fn create_violations(
    empty_list_vars_map: &EmptyListVarsMap,
    direct_copy_set: &HashSet<VarId>,
) -> Vec<Detection> {
    empty_list_vars_map
        .iter()
        .filter(|&(var_id, _)| direct_copy_set.contains(var_id))
        .map(|(_, (var_name, span))| {
            Detection::from_global_span(
                format!(
                    "Variable '{var_name}' is initialized as empty list and filled by copying \
                     items unchanged"
                ),
                *span,
            )
            .with_primary_label("empty list initialization")
        })
        .collect()
}

fn extract_empty_list_vars(expr: &Expression, context: &LintContext) -> Vec<EmptyListVar> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    if !call.is_call_to_command("mut", context) {
        return vec![];
    }

    let Some(var_arg) = call.arguments.first() else {
        return vec![];
    };
    let (Argument::Positional(var_expr) | Argument::Unknown(var_expr)) = var_arg else {
        return vec![];
    };

    let Expr::VarDecl(var_id) = &var_expr.expr else {
        return vec![];
    };

    let Some(init_arg) = call.arguments.get(1) else {
        return vec![];
    };
    let (Argument::Positional(init_expr) | Argument::Unknown(init_expr)) = init_arg else {
        return vec![];
    };

    let is_empty_list = match &init_expr.expr {
        Expr::List(items) => items.is_empty(),
        Expr::Block(block_id) => context
            .working_set
            .get_block(*block_id)
            .is_empty_list_block(),
        _ => false,
    };

    if is_empty_list {
        let var_name = var_expr.span_text(context);
        vec![(*var_id, var_name.to_string(), expr.span)]
    } else {
        vec![]
    }
}

fn is_direct_copy_for_loop(
    call: &Call,
    context: &LintContext,
) -> Option<(String, nu_protocol::BlockId)> {
    let loop_var_name = call.loop_var_from_for(context)?;
    let iter_expr = call.get_for_loop_iterator()?;
    let block_id = call.get_for_loop_body()?;

    (is_literal_list(iter_expr) && !has_transformation_or_filter(block_id, context, &loop_var_name))
        .then_some((loop_var_name, block_id))
}

fn extract_direct_copy_patterns(expr: &Expression, context: &LintContext) -> DirectCopyVars {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    if !call.is_call_to_command("for", context) {
        return vec![];
    }

    if let Some((_, block_id)) = is_direct_copy_for_loop(call, context) {
        context
            .working_set
            .get_block(block_id)
            .extract_assigned_vars()
    } else {
        vec![]
    }
}

fn extract_patterns(expr: &Expression, context: &LintContext) -> AnalysisPattern {
    (
        extract_empty_list_vars(expr, context),
        extract_direct_copy_patterns(expr, context),
    )
}

struct UnnecessaryAccumulate;

impl DetectFix for UnnecessaryAccumulate {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "unnecessary_accumulate"
    }

    fn short_description(&self) -> &'static str {
        "Redundant accumulator pattern: can be simplified"
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::UNNECESSARY]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut patterns: Vec<AnalysisPattern> = Vec::new();

        context.ast.flat_map(
            context.working_set,
            &|expr| vec![extract_patterns(expr, context)],
            &mut patterns,
        );

        let empty_list_vars: EmptyListVarsMap = patterns
            .iter()
            .flat_map(|(empty_vars, _)| empty_vars.iter())
            .map(|(id, name, span)| (*id, (name.clone(), *span)))
            .collect();

        let direct_copy_set: HashSet<VarId> = patterns
            .iter()
            .flat_map(|(_, direct_copies)| direct_copies.iter())
            .copied()
            .collect();

        let violations = create_violations(&empty_list_vars, &direct_copy_set);

        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &UnnecessaryAccumulate;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
