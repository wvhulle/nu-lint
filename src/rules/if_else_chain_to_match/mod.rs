use core::iter;

use nu_protocol::{
    Span,
    ast::{Call, Comparison, Expr, Expression, Operator},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

fn extract_compared_variable(expr: &Expression, context: &LintContext) -> Option<String> {
    let Expr::BinaryOp(left, op, right) = &expr.expr else {
        return None;
    };

    let Expr::Operator(Operator::Comparison(Comparison::Equal | Comparison::NotEqual)) = &op.expr
    else {
        return None;
    };

    if let Some(var_name) = left.extract_variable_name(context) {
        return Some(var_name);
    }

    if let Expr::FullCellPath(cell_path) = &left.expr {
        return Some(context.expr_text(&cell_path.head).to_string());
    }

    if let Some(var_name) = right.extract_variable_name(context) {
        return Some(var_name);
    }

    if let Expr::FullCellPath(cell_path) = &right.expr {
        Some(context.expr_text(&cell_path.head).to_string())
    } else {
        None
    }
}

fn extract_comparison_value(expr: &Expression, context: &LintContext) -> Option<String> {
    let Expr::BinaryOp(left, _op, right) = &expr.expr else {
        return None;
    };

    if left.extract_variable_name(context).is_some() || matches!(&left.expr, Expr::FullCellPath(_))
    {
        Some(context.expr_text(right).to_string())
    } else {
        Some(context.expr_text(left).to_string())
    }
}

/// Semantic fix data: stores all information needed to generate the match
/// expression
pub struct FixData {
    /// Span of the entire if-else-if chain to replace
    call_span: Span,
    /// Variable name being compared
    compared_var: String,
    /// Collected branches with pattern and body text
    branches: Vec<MatchBranch>,
    /// Optional final else body
    final_else: Option<String>,
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
    pattern: String,
    body: String,
}

/// Result of iterating through an if-else-if chain
enum ChainIterResult {
    /// A branch with pattern and body
    Branch(MatchBranch),
    /// The final else clause
    FinalElse(String),
}

/// Iterator over if-else-if chain branches
struct ChainIterator<'a> {
    current: Option<&'a Call>,
    context: &'a LintContext<'a>,
    final_else_pending: Option<String>,
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
        let pattern = call
            .get_first_positional_arg()
            .and_then(|arg| extract_comparison_value(arg, self.context))?;

        let body = call
            .get_positional_arg(1)
            .map(|arg| self.context.span_text(arg.span).trim().to_string())?;

        let branch = MatchBranch { pattern, body };

        // Check for else/else-if branch
        match call.get_else_branch() {
            Some((true, else_expr)) => {
                // else-if: advance to next call
                if let Expr::Call(next_call) = &else_expr.expr {
                    self.current = Some(next_call);
                } else {
                    self.current = None;
                }
                Some(ChainIterResult::Branch(branch))
            }
            Some((false, else_expr)) => {
                // Final else: store it for next iteration, return branch now
                self.current = None;
                self.final_else_pending =
                    Some(self.context.span_text(else_expr.span).trim().to_string());
                Some(ChainIterResult::Branch(branch))
            }
            None => {
                // No else branch: done after this
                self.current = None;
                Some(ChainIterResult::Branch(branch))
            }
        }
    }
}

/// Collects all branches from an if-else-if chain
fn collect_chain_branches(
    call: &Call,
    context: &LintContext,
) -> (Vec<MatchBranch>, Option<String>) {
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
    compared_var: &str,
    context: &LintContext,
) -> ChainAnalysis {
    let mut current_call = first_call;
    let mut chain_length = 2; // First if + one else-if

    // Collect all subsequent else-if branches
    let subsequent_branches = iter::from_fn(|| {
        // Check if current branch compares the same variable
        let compares_same_var = current_call
            .get_first_positional_arg()
            .and_then(|arg| extract_compared_variable(arg, context))
            .is_some_and(|var| var == compared_var);

        // Try to get the next else-if branch
        let (is_else_if, else_expr) = current_call.get_else_branch()?;
        if !is_else_if {
            return None;
        }

        let Expr::Call(next_call) = &else_expr.expr else {
            return None;
        };

        next_call.is_call_to_command("if", context).then(|| {
            current_call = next_call;
            chain_length += 1;
            compares_same_var
        })
    })
    .collect::<Vec<_>>();

    ChainAnalysis {
        length: chain_length,
        consistent_variable: subsequent_branches.iter().all(|&same| same),
    }
}

/// Analyze an if-call and its else branch to detect if-else-if chains
fn analyze_if_chain(call: &Call, context: &LintContext) -> Option<(Detection, FixData)> {
    // Get the condition expression and check if it compares a variable
    let compared_var = call
        .get_first_positional_arg()
        .and_then(|arg| extract_compared_variable(arg, context))?;

    // Verify this is an else-if chain (not just a final else)
    let (is_else_if, else_expr) = call.get_else_branch()?;
    if !is_else_if {
        return None;
    }

    // Extract the nested if call from the else-if
    let Expr::Call(nested_call) = &else_expr.expr else {
        return None;
    };

    nested_call
        .is_call_to_command("if", context)
        .then_some(())?;

    // Analyze chain properties
    let analysis = walk_if_else_chain(nested_call, &compared_var, context);

    // Only flag chains with 3+ branches (at least 2 else-if)
    (analysis.length >= 3).then_some(())?;

    // Create appropriate violation message
    let first_branch_span = call.get_first_positional_arg().map(|arg| arg.span);
    let violation = if analysis.consistent_variable {
        let mut v = Detection::from_global_span(
            format!(
                "If-else-if chain comparing '{compared_var}' to different values - consider using \
                 'match'"
            ),
            call.head,
        )
        .with_primary_label("if keyword");
        if let Some(cond_span) = first_branch_span {
            v = v.with_extra_label(format!("comparing '{compared_var}'"), cond_span);
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
    let (branches, final_else) = if analysis.consistent_variable {
        collect_chain_branches(call, context)
    } else {
        (vec![], None)
    };

    let fix_data = FixData {
        call_span: call.span(),
        compared_var,
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
        "Use 'match' for value-based branching instead of if-else-if chains"
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
                && call.is_call_to_command("if", ctx)
            {
                return analyze_if_chain(call, ctx).into_iter().collect();
            }
            vec![]
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // Only generate fix if we have branch data (consistent variable case)
        if fix_data.branches.is_empty() {
            return None;
        }

        // Build match arms from stored branch data
        let match_arms = fix_data
            .branches
            .iter()
            .map(|branch| format!("    {} => {},", branch.pattern, branch.body))
            .chain(
                fix_data
                    .final_else
                    .iter()
                    .map(|body| format!("    _ => {body}")),
            )
            .collect::<Vec<_>>()
            .join("\n");

        let match_text = format!("match {} {{\n{match_arms}\n}}", fix_data.compared_var);

        Some(Fix {
            explanation: format!("Convert to match expression on {}", fix_data.compared_var).into(),
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
