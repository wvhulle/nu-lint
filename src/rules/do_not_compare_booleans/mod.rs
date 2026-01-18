use lsp_types::DiagnosticTag;
use nu_protocol::{
    Span,
    ast::{Comparison, Expr, Expression, Operator},
};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Fix data containing the span and replacement info
pub struct FixData {
    /// The span of the entire comparison expression
    full_expr: Span,
    /// The span of the non-boolean operand (the variable/expression to keep)
    operand: Span,
    /// Whether the fix needs negation (not $x)
    needs_negation: bool,
}

/// Check if an expression is a boolean literal and return its value
const fn is_bool_literal(expr: &Expression) -> Option<bool> {
    match &expr.expr {
        Expr::Bool(value) => Some(*value),
        _ => None,
    }
}

/// Analyze a binary comparison with a boolean literal.
const fn analyze_bool_comparison(
    left: &Expression,
    right: &Expression,
    is_equal: bool,
) -> Option<(Span, bool)> {
    // Check if left is bool literal
    if let Some(bool_val) = is_bool_literal(left) {
        // `true == $x` or `false == $x` etc.
        let needs_negation = if is_equal {
            !bool_val // `true ==` keeps as-is, `false ==` needs negation
        } else {
            bool_val // `true !=` needs negation, `false !=` keeps as-is
        };
        return Some((right.span, needs_negation));
    }

    // Check if right is bool literal
    if let Some(bool_val) = is_bool_literal(right) {
        // `$x == true` or `$x == false` etc.
        let needs_negation = if is_equal {
            !bool_val // `== true` keeps as-is, `== false` needs negation
        } else {
            bool_val // `!= true` needs negation, `!= false` keeps as-is
        };
        return Some((left.span, needs_negation));
    }

    None
}

fn detect_redundant_bool_comparison(
    expr: &Expression,
    context: &LintContext,
) -> Vec<(Detection, FixData)> {
    let Expr::BinaryOp(left, op, right) = &expr.expr else {
        return vec![];
    };

    // Check if this is an equality/inequality comparison
    let is_equal = match &op.expr {
        Expr::Operator(Operator::Comparison(Comparison::Equal)) => true,
        Expr::Operator(Operator::Comparison(Comparison::NotEqual)) => false,
        _ => return vec![],
    };

    // Analyze if one side is a boolean literal
    let Some((operand_span, needs_negation)) = analyze_bool_comparison(left, right, is_equal)
    else {
        return vec![];
    };

    let bool_literal = if is_bool_literal(left).is_some() {
        context.span_text(left.span)
    } else {
        context.span_text(right.span)
    };

    let comparison_op = if is_equal { "==" } else { "!=" };

    let detection = Detection::from_global_span(
        format!(
            "Comparing with `{bool_literal}` using `{comparison_op}` is redundant. Use the \
             boolean value directly{}",
            if needs_negation { " with `not`" } else { "" }
        ),
        expr.span,
    )
    .with_primary_label("redundant boolean comparison");

    let fix_data = FixData {
        full_expr: expr.span,
        operand: operand_span,
        needs_negation,
    };

    vec![(detection, fix_data)]
}

struct RedundantBooleanComparison;

impl DetectFix for RedundantBooleanComparison {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "do_not_compare_booleans"
    }

    fn short_description(&self) -> &'static str {
        "Redundant comparison with boolean literal"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/types_of_data.html#booleans")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::UNNECESSARY]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| detect_redundant_bool_comparison(expr, ctx))
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let operand_text = context.span_text(fix_data.operand);

        let replacement_text = if fix_data.needs_negation {
            format!("(not {operand_text})")
        } else {
            operand_text.to_string()
        };

        Some(Fix {
            explanation: "simplify".into(),
            replacements: vec![Replacement::new(fix_data.full_expr, replacement_text)],
        })
    }
}

pub static RULE: &dyn Rule = &RedundantBooleanComparison;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
