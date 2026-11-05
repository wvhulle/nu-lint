use nu_lint_analysis::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Span, VarId,
    ast::{Call, Comparison, Expr, Expression, Operator, PathMember},
};

const fn is_compared_operand(operand: &Expression) -> bool {
    matches!(operand.expr, Expr::Var(_) | Expr::FullCellPath(_))
}

fn variable_path_of_operand(operand: &Expression) -> Option<(VarId, &[PathMember])> {
    match &operand.expr {
        Expr::Var(var_id) => Some((*var_id, &[])),
        Expr::FullCellPath(cell_path) => match cell_path.head.expr {
            Expr::Var(var_id) => Some((var_id, cell_path.tail.as_slice())),
            _ => None,
        },
        _ => None,
    }
}

fn extract_compared_operand(expr: &Expression) -> Option<&Expression> {
    let Expr::BinaryOp(left, op, right) = &expr.expr else {
        return None;
    };

    let Expr::Operator(Operator::Comparison(Comparison::Equal | Comparison::NotEqual)) = &op.expr
    else {
        return None;
    };

    [left, right]
        .into_iter()
        .find(|operand| is_compared_operand(operand))
        .map(AsRef::as_ref)
}

fn extract_comparison_value(expr: &Expression, context: &LintContext) -> Option<Span> {
    let Expr::BinaryOp(left, _op, right) = &expr.expr else {
        return None;
    };

    if left.extract_variable_name(context).is_some() || matches!(&left.expr, Expr::FullCellPath(_))
    {
        Some(right.span)
    } else {
        Some(left.span)
    }
}

/// Semantic fix data: stores all information needed to generate the match
/// expression
pub struct FixData {
    /// Span of the entire if-else-if chain to replace
    call_span: Span,
    compared_operand_span: Span,
    branches: Vec<MatchBranch>,
    /// Optional final else body
    final_else: Option<Span>,
}

/// Properties of an if-else-if chain
struct ChainAnalysis {
    /// Number of branches in the chain (including initial if)
    length: usize,
    /// Whether all branches compare the same variable
    consistent_variable: bool,
}

/// Represents a single branch in the if-else-if chain
struct MatchBranch {
    pattern_span: Span,
    body_span: Span,
}

/// Result of iterating through an if-else-if chain
enum ChainIterResult {
    /// A branch with pattern and body
    Branch(MatchBranch),
    /// The final else clause
    FinalElse(Span),
}

/// Iterator over if-else-if chain branches
struct ChainIterator<'a> {
    current: Option<&'a Call>,
    context: &'a LintContext<'a>,
    final_else_pending: Option<Span>,
}

impl<'a> ChainIterator<'a> {
    const fn new(call: &'a Call, context: &'a LintContext<'a>) -> Self {
        Self {
            current: Some(call),
            context,
            final_else_pending: None,
        }
    }
}

impl Iterator for ChainIterator<'_> {
    type Item = ChainIterResult;

    fn next(&mut self) -> Option<Self::Item> {
        // If we have a pending final else, return it
        if let Some(final_else) = self.final_else_pending.take() {
            return Some(ChainIterResult::FinalElse(final_else));
        }

        let call = self.current?;

        // Extract pattern and body from current branch
        let pattern_span = call
            .get_first_positional_arg()
            .and_then(|arg| extract_comparison_value(arg, self.context))?;

        let body_span = call.get_positional_arg(1).map(|arg| arg.span)?;

        let branch = MatchBranch {
            pattern_span,
            body_span,
        };

        // Check for else/else-if branch
        match call.get_else_branch() {
            Some((true, else_expr)) => {
                self.current = match &else_expr.expr {
                    Expr::Call(next_call) => Some(next_call),
                    _ => None,
                };
            }
            Some((false, final_else_expr)) => {
                self.current = None;
                self.final_else_pending = Some(final_else_expr.span);
            }
            None => {
                self.current = None;
            }
        }
        Some(ChainIterResult::Branch(branch))
    }
}

/// Collects all branches from an if-else-if chain
fn collect_chain_branches(call: &Call, context: &LintContext) -> (Vec<MatchBranch>, Option<Span>) {
    let mut branches = Vec::new();
    let mut final_else = None;

    for result in ChainIterator::new(call, context) {
        match result {
            ChainIterResult::Branch(branch) => branches.push(branch),
            ChainIterResult::FinalElse(else_body) => final_else = Some(else_body),
        }
    }

    (branches, final_else)
}

/// Walks the else-if chain and analyzes its properties
fn walk_if_else_chain(
    first_call: &Call,
    compared_path: Option<(VarId, &[PathMember])>,
    context: &LintContext,
) -> ChainAnalysis {
    let mut current_call = first_call;
    let mut chain_length = 2;
    let mut consistent_variable = true;

    loop {
        consistent_variable &= current_call
            .get_first_positional_arg()
            .and_then(extract_compared_operand)
            .and_then(variable_path_of_operand)
            .is_some_and(|branch_path| Some(branch_path) == compared_path);

        match next_else_if_call(current_call, context) {
            Some(next_call) => {
                current_call = next_call;
                chain_length += 1;
            }
            None => break,
        }
    }

    ChainAnalysis {
        length: chain_length,
        consistent_variable,
    }
}

fn next_else_if_call<'a>(call: &'a Call, context: &LintContext) -> Option<&'a Call> {
    let (is_else_if, else_expr) = call.get_else_branch()?;
    let Expr::Call(next_call) = &else_expr.expr else {
        return None;
    };
    (is_else_if && next_call.builtin_name(context) == Some("if")).then_some(next_call)
}

fn is_equality_with_literal(condition: &Expression) -> bool {
    let Expr::BinaryOp(left, comparison_op, right) = &condition.expr else {
        return false;
    };
    let is_literal = |operand: &Expression| {
        matches!(
            operand.expr,
            Expr::Bool(_)
                | Expr::Float(_)
                | Expr::Int(_)
                | Expr::Nothing
                | Expr::RawString(_)
                | Expr::String(_)
        )
    };
    matches!(
        comparison_op.expr,
        Expr::Operator(Operator::Comparison(Comparison::Equal))
    ) && (is_literal(left) || is_literal(right))
}

fn chain_compares_literals_only(first_call: &Call, context: &LintContext) -> bool {
    let mut current_call = Some(first_call);
    while let Some(call) = current_call {
        if !call
            .get_first_positional_arg()
            .is_some_and(is_equality_with_literal)
        {
            return false;
        }
        current_call = next_else_if_call(call, context);
    }
    true
}

/// Analyze an if-call and its else branch to detect if-else-if chains
fn analyze_if_chain(call: &Call, context: &LintContext) -> Option<(Detection, FixData)> {
    // Get the condition expression and check if it compares a variable
    let compared_operand = call
        .get_first_positional_arg()
        .and_then(extract_compared_operand)?;
    let compared_operand_span = compared_operand.span;
    let compared_operand_text = context.span_text(compared_operand_span);

    // Verify this is an else-if chain (not just a final else)
    let (is_else_if, else_expr) = call.get_else_branch()?;
    if !is_else_if {
        return None;
    }

    // Extract the nested if call from the else-if
    let Expr::Call(nested_call) = &else_expr.expr else {
        return None;
    };

    (nested_call.builtin_name(context) == Some("if")).then_some(())?;

    // Analyze chain properties
    let analysis = walk_if_else_chain(
        nested_call,
        variable_path_of_operand(compared_operand),
        context,
    );

    // Only flag chains with 3+ branches (at least 2 else-if)
    (analysis.length >= 3).then_some(())?;

    // Create appropriate violation message
    let first_branch_span = call.get_first_positional_arg().map(|arg| arg.span);
    let violation = if analysis.consistent_variable {
        let mut v = Detection::from_global_span(
            format!(
                "If-else-if chain comparing '{compared_operand_text}' to different values - \
                 consider using 'match'"
            ),
            call.head,
        )
        .with_primary_label("if keyword");
        if let Some(cond_span) = first_branch_span {
            v = v.with_extra_label(format!("comparing '{compared_operand_text}'"), cond_span);
        }
        v
    } else {
        Detection::from_global_span(
            "Long if-else-if chain - consider using 'match' for clearer branching",
            call.head,
        )
        .with_primary_label("start of chain")
    };

    // Collect branch data for fix generation (only if consistent variable)
    let (branches, final_else) =
        if analysis.consistent_variable && chain_compares_literals_only(call, context) {
            collect_chain_branches(call, context)
        } else {
            (vec![], None)
        };

    let fix_data = FixData {
        call_span: call.span(),
        compared_operand_span,
        branches,
        final_else,
    };

    Some((violation, fix_data))
}

struct ReplaceIfElseChainWithMatch;

impl DetectFix for ReplaceIfElseChainWithMatch {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "if_else_chain_to_match"
    }

    fn short_description(&self) -> &'static str {
        "Use `match` for value-based branching instead of if-else-if chains"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/match.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            if let Expr::Call(call) = &expr.expr
                && call.builtin_name(ctx) == Some("if")
            {
                return analyze_if_chain(call, ctx).into_iter().collect();
            }
            vec![]
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // Only generate fix if we have branch data (consistent variable case)
        if fix_data.branches.is_empty() {
            return None;
        }

        // Build match arms from stored branch data
        let match_arms = fix_data
            .branches
            .iter()
            .map(|branch| {
                format!(
                    "    {} => {},",
                    context.span_text(branch.pattern_span),
                    context.span_text(branch.body_span)
                )
            })
            .chain(
                fix_data
                    .final_else
                    .iter()
                    .map(|body_span| format!("    _ => {}", context.span_text(*body_span))),
            )
            .collect::<Vec<_>>()
            .join("\n");

        let compared_operand_text = context.span_text(fix_data.compared_operand_span);
        let match_text = format!("match {compared_operand_text} {{\n{match_arms}\n}}");

        Some(Fix {
            explanation: format!("Convert to match expression on {compared_operand_text}").into(),
            replacements: vec![Replacement::new(fix_data.call_span, match_text)],
        })
    }
}

pub static RULE: &dyn Rule = &ReplaceIfElseChainWithMatch;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
