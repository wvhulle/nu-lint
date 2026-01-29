use std::collections::{HashMap, HashSet};

use nu_protocol::ast::{Argument, Block, Call, Expr, Expression, Operator, Traverse};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn contains_loop_var_append(expr: &Expression, context: &LintContext, loop_var_name: &str) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let decl_name = call.get_call_name(context);
            log::trace!("Found call to: {decl_name}");

            if decl_name == "append"
                && let Some(arg_expr) = call.get_first_positional_arg()
            {
                let is_loop_var = arg_expr
                    .extract_variable_name(context)
                    .is_some_and(|name| name == loop_var_name);
                log::trace!("Append argument is loop var: {is_loop_var}");
                return is_loop_var;
            }
            false
        }
        Expr::FullCellPath(cell_path) => {
            contains_loop_var_append(&cell_path.head, context, loop_var_name)
        }
        _ => expr.extract_block_id().is_some_and(|block_id| {
            let block = context.working_set.get_block(block_id);
            block
                .pipelines
                .iter()
                .flat_map(|p| &p.elements)
                .any(|elem| contains_loop_var_append(&elem.expr, context, loop_var_name))
        }),
    }
}

fn has_append_without_transformation(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);
    log::trace!(
        "Checking append pattern: block has {} elements",
        block.all_elements().len()
    );

    block
        .all_elements()
        .iter()
        .any(|elem| matches_append_assignment(&elem.expr, context, loop_var_name))
}

fn matches_append_assignment(
    expr: &Expression,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let Expr::BinaryOp(_lhs, op, rhs) = &expr.expr else {
        return false;
    };

    if !matches!(op.expr, Expr::Operator(Operator::Assignment(_))) {
        return false;
    }

    let result = contains_loop_var_append(rhs, context, loop_var_name);
    log::trace!("Assignment RHS contains loop var append: {result}");
    result
}

fn is_filtering_only_pattern(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);
    log::trace!(
        "Checking filtering pattern: block has {} pipelines",
        block.pipelines.len()
    );

    let Some(pipeline) = block.pipelines.first() else {
        log::trace!("Block has no pipelines");
        return false;
    };

    if pipeline.elements.len() != 1 {
        log::trace!(
            "Pipeline has {} elements, expected 1",
            pipeline.elements.len()
        );
        return false;
    }

    let elem = &pipeline.elements[0];

    let Expr::Call(call) = &elem.expr.expr else {
        log::trace!("Element is not a Call");
        return false;
    };

    if !call.is_call_to_command("if", context) {
        log::trace!("Command is not 'if'");
        return false;
    }

    log::trace!("Found 'if' with {} arguments", call.arguments.len());

    if call.arguments.len() != 2 {
        log::trace!(
            "if statement has {} arguments, expected 2 (has else clause)",
            call.arguments.len()
        );
        return false;
    }

    let Some(then_block_expr) = call.get_positional_arg(1) else {
        log::trace!("No then-block argument");
        return false;
    };

    let Some(then_block_id) = then_block_expr.extract_block_id() else {
        log::trace!("Then-block is not a Block or Closure");
        return false;
    };

    let result = has_append_without_transformation(then_block_id, context, loop_var_name);
    log::trace!("has_append_without_transformation: {result}");
    result
}

fn extract_empty_list_vars(
    expr: &Expression,
    context: &LintContext,
) -> Vec<(nu_protocol::VarId, String, nu_protocol::Span)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    let decl_name = call.get_call_name(context);
    log::trace!("Checking call to: {decl_name}");

    if decl_name != "mut" {
        return vec![];
    }

    log::trace!("Found 'mut' declaration");

    let Some((var_id, var_name, _var_span)) = call.extract_variable_declaration(context) else {
        log::trace!("Could not extract variable declaration");
        return vec![];
    };

    let Some(init_expr) = call.get_positional_arg(1) else {
        log::trace!("No init argument");
        return vec![];
    };

    log::trace!("Init expr type: {:?}", init_expr.expr);

    let is_empty_list = if init_expr.is_empty_list() {
        true
    } else if let Some(block_id) = init_expr.extract_block_id() {
        context
            .working_set
            .get_block(block_id)
            .is_empty_list_block()
    } else {
        false
    };

    log::trace!("is_empty_list: {is_empty_list}");

    if is_empty_list {
        log::trace!("Found empty list var: {var_name} (id: {var_id:?})");
        vec![(var_id, var_name, expr.span)]
    } else {
        vec![]
    }
}

fn extract_assigned_var_ids_from_if(
    if_call: &Call,
    context: &LintContext,
) -> Vec<nu_protocol::VarId> {
    let mut var_ids = Vec::new();

    let Some(Argument::Positional(then_expr) | Argument::Unknown(then_expr)) =
        if_call.arguments.get(1)
    else {
        return var_ids;
    };

    let (Expr::Block(then_block_id) | Expr::Closure(then_block_id)) = &then_expr.expr else {
        return var_ids;
    };

    let then_block = context.working_set.get_block(*then_block_id);

    for p in &then_block.pipelines {
        for e in &p.elements {
            let Expr::BinaryOp(_lhs, op, _rhs) = &e.expr.expr else {
                continue;
            };

            let is_assignment = matches!(op.expr, Expr::Operator(Operator::Assignment(_)));
            if !is_assignment {
                continue;
            }

            if let Some(id) = e.expr.extract_assigned_variable() {
                var_ids.push(id);
            }
        }
    }

    var_ids
}

fn extract_filtering_vars(expr: &Expression, context: &LintContext) -> Vec<nu_protocol::VarId> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    if !call.is_call_to_command("for", context) {
        return vec![];
    }

    log::trace!("Found 'for' loop");

    let Some(loop_var_name) = call.loop_var_from_for(context) else {
        log::trace!("Could not get loop var name");
        return vec![];
    };

    let Some(block_expr) = call.arguments.last().and_then(|arg| match arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => Some(expr),
        _ => None,
    }) else {
        log::trace!("No block argument");
        return vec![];
    };

    let Some(block_id) = block_expr.extract_block_id() else {
        log::trace!("Loop body is not a block or closure");
        return vec![];
    };

    if !is_filtering_only_pattern(block_id, context, &loop_var_name) {
        log::trace!("Not a filtering-only pattern");
        return vec![];
    }

    log::trace!("Found filtering pattern, extracting assigned variables");

    let block = context.working_set.get_block(block_id);
    extract_var_ids_from_if_statements(block, context)
}

fn extract_var_ids_from_if_statements(
    block: &Block,
    context: &LintContext,
) -> Vec<nu_protocol::VarId> {
    let mut var_ids = Vec::new();

    for pipeline in &block.pipelines {
        for elem in &pipeline.elements {
            let Expr::Call(if_call) = &elem.expr.expr else {
                continue;
            };

            let if_decl = context.working_set.get_decl(if_call.decl_id).name();
            if if_decl != "if" {
                continue;
            }

            var_ids.extend(extract_assigned_var_ids_from_if(if_call, context));
        }
    }

    var_ids
}

struct FilterCollectWithWhere;

impl DetectFix for FilterCollectWithWhere {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "for_filter_to_where"
    }

    fn short_description(&self) -> &'static str {
        "Use 'where' filter instead of for loop with if and append"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/where.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut empty_list_vars = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| extract_empty_list_vars(expr, context),
            &mut empty_list_vars,
        );

        log::trace!("Found {} empty list vars", empty_list_vars.len());

        let empty_list_vars_map: HashMap<nu_protocol::VarId, (String, nu_protocol::Span)> =
            empty_list_vars
                .into_iter()
                .map(|(id, name, span)| (id, (name, span)))
                .collect();

        let mut filtering_vars = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| extract_filtering_vars(expr, context),
            &mut filtering_vars,
        );

        log::trace!("Found {} filtering vars", filtering_vars.len());

        let filtering_set: HashSet<nu_protocol::VarId> = filtering_vars.into_iter().collect();

        let mut violations = Vec::new();
        for (var_id, (var_name, span)) in &empty_list_vars_map {
            if filtering_set.contains(var_id) {
                log::trace!("Creating violation for var '{var_name}'");
                let violation = Detection::from_global_span(
                    format!(
                        "Variable '{var_name}' accumulates filtered items - use 'where' instead"
                    ),
                    *span,
                )
                .with_primary_label("accumulator variable");
                violations.push(violation);
            }
        }

        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &FilterCollectWithWhere;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
