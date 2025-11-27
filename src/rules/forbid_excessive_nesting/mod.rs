use nu_protocol::ast::{
    Block, Call, Expr, Expression, ListItem, MatchPattern, Pattern, Pipeline, RecordItem,
};

use crate::{ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation};

const MAX_NESTING_DEPTH: usize = 4;

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    for block_idx in 0..context.working_set.num_blocks() {
        let block_id = nu_protocol::BlockId::new(block_idx);
        let block = context.working_set.get_block(block_id);
        check_block_nesting(block, 0, &mut violations, context);
    }

    violations
}

fn check_block_nesting(
    block: &Block,
    current_depth: usize,
    violations: &mut Vec<Violation>,
    context: &LintContext,
) {
    for pipeline in &block.pipelines {
        check_pipeline_nesting(pipeline, current_depth, violations, context);
    }
}

fn check_pipeline_nesting(
    pipeline: &Pipeline,
    current_depth: usize,
    violations: &mut Vec<Violation>,
    context: &LintContext,
) {
    for element in &pipeline.elements {
        check_expr_nesting(&element.expr, current_depth, violations, context);
    }
}

fn check_control_flow_call(
    call: &Call,
    command_name: &str,
    current_depth: usize,
    violations: &mut Vec<Violation>,
    context: &LintContext,
) {
    let new_depth = current_depth + 1;

    match command_name {
        "if" => {
            // Check condition
            if let Some(condition) = call.get_positional_arg(0) {
                check_expr_nesting(condition, current_depth, violations, context);
            }
            // Check then block
            if let Some(then_expr) = call.get_positional_arg(1) {
                check_expr_nesting(then_expr, new_depth, violations, context);
            }
            // Check else branch (stays at same depth as if)
            if let Some((_is_else_if, else_expr)) = call.get_else_branch() {
                check_expr_nesting(else_expr, current_depth, violations, context);
            }
        }
        "for" => {
            // Check iterator expression
            if let Some(iterable) = call.get_for_loop_iterator() {
                check_expr_nesting(iterable, current_depth, violations, context);
            }
            // Check loop body
            if let Some(body_block_id) = call.get_for_loop_body() {
                let body_block = context.working_set.get_block(body_block_id);
                check_block_nesting(body_block, new_depth, violations, context);
            }
        }
        "while" => {
            // Check condition
            if let Some(condition) = call.get_positional_arg(0) {
                check_expr_nesting(condition, current_depth, violations, context);
            }
            // Check body
            if let Some(body_expr) = call.get_positional_arg(1) {
                check_expr_nesting(body_expr, new_depth, violations, context);
            }
        }
        "match" => {
            // Check matched expression
            if let Some(match_expr) = call.get_positional_arg(0) {
                check_expr_nesting(match_expr, current_depth, violations, context);
            }
            // Check match arms
            check_match_arms(call, new_depth, violations, context);
        }
        "try" => {
            // Check try body
            if let Some(try_body) = call.get_positional_arg(0) {
                check_expr_nesting(try_body, new_depth, violations, context);
            }
            // Check catch block if present
            if let Some(catch_expr) = call.get_named_arg_expr("catch") {
                check_expr_nesting(catch_expr, new_depth, violations, context);
            }
        }
        _ => {}
    }
}

fn check_match_arms(
    call: &Call,
    depth: usize,
    violations: &mut Vec<Violation>,
    context: &LintContext,
) {
    if let Some(arms_expr) = call.get_positional_arg(1)
        && let Expr::MatchBlock(arms) = &arms_expr.expr
    {
        for (pattern, arm_expr) in arms {
            check_pattern_nesting(pattern, depth - 1, violations, context);
            check_expr_nesting(arm_expr, depth, violations, context);
        }
    }
}

fn check_call_arguments(
    call: &Call,
    depth: usize,
    violations: &mut Vec<Violation>,
    context: &LintContext,
) {
    for expr in call.all_arg_expressions() {
        check_expr_nesting(expr, depth, violations, context);
    }
}

fn check_expr_nesting(
    expr: &Expression,
    current_depth: usize,
    violations: &mut Vec<Violation>,
    context: &LintContext,
) {
    match &expr.expr {
        Expr::Call(call) => {
            if call.is_control_flow_command(context) {
                let command_name = call.get_call_name(context);
                check_control_flow_call(call, &command_name, current_depth, violations, context);
            } else {
                check_call_arguments(call, current_depth, violations, context);
            }
        }

        // Blocks and closures contain code but don't add nesting
        // (they represent function boundaries, not control flow)
        Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            if current_depth > MAX_NESTING_DEPTH {
                violations.push(create_violation(expr.span, current_depth));
            }
            check_block_nesting(block, current_depth, violations, context);
        }

        // MatchBlock as an expression (not in a call)
        Expr::MatchBlock(patterns) => {
            let new_depth = current_depth + 1;
            for (pattern, arm_expr) in patterns {
                check_pattern_nesting(pattern, current_depth, violations, context);
                if new_depth > MAX_NESTING_DEPTH {
                    violations.push(create_violation(arm_expr.span, new_depth));
                }
                check_expr_nesting(arm_expr, new_depth, violations, context);
            }
        }

        // Recurse into other expression types
        Expr::FullCellPath(cell_path) => {
            check_expr_nesting(&cell_path.head, current_depth, violations, context);
        }

        Expr::BinaryOp(left, op, right) => {
            check_expr_nesting(left, current_depth, violations, context);
            check_expr_nesting(op, current_depth, violations, context);
            check_expr_nesting(right, current_depth, violations, context);
        }

        Expr::UnaryNot(inner) => {
            check_expr_nesting(inner, current_depth, violations, context);
        }

        Expr::List(items) => {
            for item in items {
                let item_expr = match item {
                    ListItem::Item(e) | ListItem::Spread(_, e) => e,
                };
                check_expr_nesting(item_expr, current_depth, violations, context);
            }
        }

        Expr::Record(fields) => {
            for field in fields {
                match field {
                    RecordItem::Pair(key, value) => {
                        check_expr_nesting(key, current_depth, violations, context);
                        check_expr_nesting(value, current_depth, violations, context);
                    }
                    RecordItem::Spread(_, record_expr) => {
                        check_expr_nesting(record_expr, current_depth, violations, context);
                    }
                }
            }
        }

        Expr::Collect(_, collect_expr) => {
            check_expr_nesting(collect_expr, current_depth, violations, context);
        }

        // Literals and other expressions don't increase nesting
        _ => {}
    }
}

fn check_pattern_nesting(
    pattern: &MatchPattern,
    current_depth: usize,
    violations: &mut Vec<Violation>,
    context: &LintContext,
) {
    match &pattern.pattern {
        Pattern::Expression(pattern_expr) => {
            check_expr_nesting(pattern_expr, current_depth, violations, context);
        }
        Pattern::List(items) => {
            for item in items {
                check_pattern_nesting(item, current_depth, violations, context);
            }
        }
        Pattern::Record(fields) => {
            for (_, field_pattern) in fields {
                check_pattern_nesting(field_pattern, current_depth, violations, context);
            }
        }
        Pattern::Or(patterns) => {
            for pat in patterns {
                check_pattern_nesting(pat, current_depth, violations, context);
            }
        }
        _ => {}
    }
}

fn create_violation(span: nu_protocol::Span, depth: usize) -> Violation {
    Violation::new(format!(
            "Code has nesting depth of {depth}, which exceeds the maximum of {MAX_NESTING_DEPTH}"
        ),
        span,
    )
    .with_help("Consider refactoring this code into smaller functions to reduce nesting depth")
}

pub const fn rule() -> Rule {
    Rule::new(
        "forbid_excessive_nesting",
        "Avoid excessive nesting (more than 4 levels deep)",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/thinking_in_nu.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
