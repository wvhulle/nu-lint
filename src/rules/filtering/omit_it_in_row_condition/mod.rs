use nu_protocol::ast::{Argument, Expr, Expression, Operator, PathMember, Traverse};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct ItFieldAccess {
    full_span: nu_protocol::Span,
    field_name: String,
}

fn is_it_variable(expr: &Expression, context: &LintContext) -> bool {
    if let Expr::Var(_var_id) = &expr.expr {
        let var_text = context.expr_text(expr);
        log::debug!("Checking if var is 'it': {var_text}");
        return var_text == "$it";
    }
    log::debug!("Not a Var expression");
    false
}

fn extract_it_field_access(expr: &Expression, context: &LintContext) -> Option<ItFieldAccess> {
    log::debug!("extract_it_field_access called");
    let Expr::FullCellPath(cell_path) = &expr.expr else {
        log::debug!("Not a FullCellPath");
        return None;
    };

    log::debug!("Is FullCellPath, checking if head is $it");
    if !is_it_variable(&cell_path.head, context) {
        log::debug!("Head is not $it");
        return None;
    }

    if cell_path.tail.len() != 1 {
        return None;
    }

    let PathMember::String {
        val: field_name, ..
    } = &cell_path.tail[0]
    else {
        return None;
    };

    Some(ItFieldAccess {
        full_span: expr.span,
        field_name: field_name.clone(),
    })
}

fn check_binary_op_for_it_field(
    expr: &Expression,
    context: &LintContext,
    violations: &mut Vec<(Detection, ItFieldAccess)>,
) {
    log::debug!("Checking expression: {:?}", expr.expr);
    match &expr.expr {
        Expr::BinaryOp(lhs, op, rhs) => {
            let is_comparison = matches!(&op.expr, Expr::Operator(Operator::Comparison(_)));

            if is_comparison {
                log::debug!("Found comparison op, checking LHS for $it.field");
                if let Some(it_access) = extract_it_field_access(lhs, context) {
                    log::debug!("Found $it.{} on LHS of comparison!", it_access.field_name);
                    let violation = Detection::from_global_span(
                        "Field name can be used directly in row condition without `$it.` prefix",
                        it_access.full_span,
                    )
                    .with_primary_label("unnecessary `$it.` prefix");

                    violations.push((violation, it_access));
                }
            }

            check_binary_op_for_it_field(lhs, context, violations);
            check_binary_op_for_it_field(rhs, context, violations);
        }
        Expr::FullCellPath(cell_path) => {
            check_binary_op_for_it_field(&cell_path.head, context, violations);
        }
        Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
            let block = context.working_set.get_block(*block_id);
            for pipeline in &block.pipelines {
                for element in &pipeline.elements {
                    check_binary_op_for_it_field(&element.expr, context, violations);
                }
            }
        }
        _ => {}
    }
}

fn check_where_row_condition(
    expr: &Expression,
    context: &LintContext,
) -> Vec<(Detection, ItFieldAccess)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    let decl = context.working_set.get_decl(call.decl_id);
    log::debug!("Found call to: {}", decl.name());

    if decl.name() != "where" {
        return vec![];
    }

    log::debug!("Found where command");

    let Some(arg_expr) = call.arguments.first() else {
        log::debug!("No arguments to where");
        return vec![];
    };

    let (Argument::Positional(arg_expr) | Argument::Unknown(arg_expr)) = arg_expr else {
        log::debug!("Argument is not positional");
        return vec![];
    };

    log::debug!("Checking argument type: {:?}", arg_expr.expr);

    let Expr::RowCondition(block_id) = &arg_expr.expr else {
        log::debug!("Not a row condition");
        return vec![];
    };

    log::debug!("Found row condition, checking for $it.field patterns");

    let block = context.working_set.get_block(*block_id);
    let mut violations = Vec::new();

    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            check_binary_op_for_it_field(&element.expr, context, &mut violations);
        }
    }

    violations
}

fn check_expression(expr: &Expression, context: &LintContext) -> Vec<(Detection, ItFieldAccess)> {
    check_where_row_condition(expr, context)
}

struct OmitItInRowCondition;

impl DetectFix for OmitItInRowCondition {
    type FixInput<'a> = ItFieldAccess;

    fn id(&self) -> &'static str {
        "omit_it_in_row_condition"
    }

    fn short_description(&self) -> &'static str {
        "Field names in 'where' row conditions don't need `$it.` prefix"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/where.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| check_expression(expr, context),
            &mut violations,
        );

        violations
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!(
                "Remove `$it.` prefix from field name `{}`",
                fix_data.field_name
            )
            .into(),
            replacements: vec![Replacement::new(
                fix_data.full_span,
                fix_data.field_name.clone(),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &OmitItInRowCondition;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
