use nu_protocol::ast::{
    Assignment, Bits, Boolean, Comparison, Expr, FindMapResult, Math, Operator, Traverse,
};

use crate::{
    ast::expression::ExpressionExt,
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Detection categories for problematic AST patterns in string interpolations
#[derive(Debug, PartialEq)]
enum ProblematicPattern {
    /// Standalone operator without operands (e.g., `(and)`, `(or)`)
    StandaloneOperator(String),
    /// External call to boolean operator (e.g., `(and y)` parsed as external
    /// command)
    ExternalBooleanOperator(String),
    /// Binary operation with literal operands (likely meant as text)
    LiteralBinaryOp(String),
}

/// Check if a string matches any Nushell operator keyword using enum types from
/// `nu_protocol`
fn is_operator_keyword(s: &str) -> bool {
    use Boolean as B;
    use Comparison as C;
    use Math as M;

    [
        B::And.as_str(),
        B::Or.as_str(),
        B::Xor.as_str(),
        M::Add.as_str(),
        M::Subtract.as_str(),
        M::Multiply.as_str(),
        M::Divide.as_str(),
        M::FloorDivide.as_str(),
        M::Modulo.as_str(),
        M::Pow.as_str(),
        M::Concatenate.as_str(),
        C::Equal.as_str(),
        C::NotEqual.as_str(),
        C::LessThan.as_str(),
        C::GreaterThan.as_str(),
        C::LessThanOrEqual.as_str(),
        C::GreaterThanOrEqual.as_str(),
        C::RegexMatch.as_str(),
        C::NotRegexMatch.as_str(),
        C::In.as_str(),
        C::NotIn.as_str(),
        C::Has.as_str(),
        C::NotHas.as_str(),
        C::StartsWith.as_str(),
        C::NotStartsWith.as_str(),
        C::EndsWith.as_str(),
        C::NotEndsWith.as_str(),
        Bits::BitOr.as_str(),
        Bits::BitXor.as_str(),
        Bits::BitAnd.as_str(),
        Bits::ShiftLeft.as_str(),
        Bits::ShiftRight.as_str(),
        Assignment::Assign.as_str(),
        Assignment::AddAssign.as_str(),
        Assignment::SubtractAssign.as_str(),
        Assignment::MultiplyAssign.as_str(),
        Assignment::DivideAssign.as_str(),
        Assignment::ConcatenateAssign.as_str(),
    ]
    .contains(&s)
}

/// Analyze an AST expression to detect problematic patterns
/// Only detects patterns that are reliably identifiable via AST structure
/// Uses `nu_protocol`'s `Traverse` trait for comprehensive AST traversal
fn analyze_ast_expression(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<ProblematicPattern> {
    // Use Traverse to check all subexpressions
    // The closure decides what to flag as problematic
    expr.find_map(context.working_set, &|sub_expr| {
        match &sub_expr.expr {
            // Binary operations - check this FIRST before looking at operators
            // This prevents us from flagging the operator child node separately
            Expr::BinaryOp(left, op, right) => handle_binary_op(left, op, right, context),

            // Standalone operators
            // These are operators NOT part of a BinaryOp (which was handled above)
            Expr::Operator(op) => {
                FindMapResult::Found(ProblematicPattern::StandaloneOperator(format!("{op}")))
            }

            Expr::UnaryNot(inner) => {
                if inner.as_ref().contains_variables(context) {
                    FindMapResult::Continue
                } else {
                    FindMapResult::Found(ProblematicPattern::LiteralBinaryOp("not".to_string()))
                }
            }

            // External calls to operators
            Expr::ExternalCall(head, _args) => analyze_external_call(head, context)
                .map_or(FindMapResult::Continue, FindMapResult::Found),

            // For all other expressions, continue traversal
            _ => FindMapResult::Continue,
        }
    })
}

/// Handle binary operation analysis separately to reduce nesting
fn handle_binary_op(
    left: &nu_protocol::ast::Expression,
    op: &nu_protocol::ast::Expression,
    right: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> FindMapResult<ProblematicPattern> {
    // Analyze the binary operation as a whole
    if let Some(p) = analyze_binary_operation(left, op, right, context) {
        // Found a problem with this binary op
        return FindMapResult::Found(p);
    }

    // Valid binary operation - manually check operands
    // to avoid Traverse visiting the operator child as standalone
    if let Some(left_problem) = analyze_ast_expression(left, context) {
        return FindMapResult::Found(left_problem);
    }
    if let Some(right_problem) = analyze_ast_expression(right, context) {
        return FindMapResult::Found(right_problem);
    }

    // All good, stop traversing this branch (don't visit operator child)
    FindMapResult::Stop
}

/// Analyze binary operations for problematic patterns
/// Only flag when both operands are literals (no variables)
fn analyze_binary_operation(
    left: &nu_protocol::ast::Expression,
    op: &nu_protocol::ast::Expression,
    right: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<ProblematicPattern> {
    let Expr::Operator(operator) = &op.expr else {
        return None;
    };

    match operator {
        // Boolean operators with literal operands are likely meant as text
        Operator::Boolean(Boolean::And | Boolean::Or)
            if !left.contains_variables(context) && !right.contains_variables(context) =>
        {
            Some(ProblematicPattern::LiteralBinaryOp(format!("{operator}")))
        }
        _ => None,
    }
}

/// Analyze external calls - detect operators used as external commands
fn analyze_external_call(
    head: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<ProblematicPattern> {
    let (Expr::GlobPattern(pattern, _) | Expr::String(pattern)) = &head.expr else {
        return None;
    };

    // Check if this string matches any operator keyword
    if !is_operator_keyword(pattern.as_str()) {
        return None;
    }

    // Verify it's not actually a known Nushell command or external command
    context
        .working_set
        .find_decl(pattern.as_bytes())
        .is_none()
        .then(|| ProblematicPattern::ExternalBooleanOperator(pattern.clone()))
}

/// Check if an interpolation expression is valid
fn is_valid_interpolation(expr: &nu_protocol::ast::Expression, context: &LintContext) -> bool {
    match &expr.expr {
        // Subexpressions: check for problematic patterns
        Expr::Subexpression(_) => analyze_ast_expression(expr, context).is_none(),

        // Cell paths: check the head
        Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
            Expr::Subexpression(_) => analyze_ast_expression(&cell_path.head, context).is_none(),
            _ => true, // Variables and other cell path heads are valid
        },

        // All other expression types are assumed valid
        _ => true,
    }
}

fn create_violation(span: nu_protocol::Span, pattern: ProblematicPattern) -> RuleViolation {
    let (message, suggestion) = match pattern {
        ProblematicPattern::StandaloneOperator(op) => (
            format!(
                "String interpolation contains standalone operator '{op}' which will cause \
                 runtime error"
            ),
            "Use a complete expression or escape as literal text: \\(...\\)".to_string(),
        ),
        ProblematicPattern::ExternalBooleanOperator(op) => (
            format!(
                "String interpolation attempts to call operator '{op}' as external command, which \
                 will cause runtime error"
            ),
            format!(
                "Use the operator in a complete expression or escape as literal text: \\({op} \
                 ...\\)"
            ),
        ),
        ProblematicPattern::LiteralBinaryOp(op) => (
            format!(
                "String interpolation contains '{op}' operation on literal values, likely \
                 intended as text"
            ),
            "Escape literal parentheses with backslashes: \\(...\\)".to_string(),
        ),
    };

    RuleViolation::new_dynamic("escape_string_interpolation_operators", message, span)
        .with_suggestion_dynamic(suggestion)
}

fn check_string_interpolation(
    exprs: &[nu_protocol::ast::Expression],
    span: nu_protocol::Span,
    context: &LintContext,
) -> Option<RuleViolation> {
    // Only check non-string expressions (i.e., interpolated expressions)
    exprs
        .iter()
        .filter(|expr| !matches!(expr.expr, Expr::String(_)))
        .find_map(|expr| {
            // Check validity - if invalid, analyze to determine specific problem
            if is_valid_interpolation(expr, context) {
                None
            } else {
                analyze_ast_expression(expr, context).map(|pattern| create_violation(span, pattern))
            }
        })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::StringInterpolation(exprs) = &expr.expr
            && let Some(violation) = check_string_interpolation(exprs, expr.span, ctx)
        {
            vec![violation]
        } else {
            vec![]
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "escape_string_interpolation_operators",
        RuleCategory::ErrorHandling,
        Severity::Error,
        "Detect reliably identifiable AST patterns in string interpolations that will cause \
         runtime errors (standalone operators, external boolean operator calls, literal boolean \
         operations)",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
