use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Call, Expr, Expression, Traverse},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct RowConditionFixData {
    /// Span of the entire closure argument (including braces): `{|x| $x > 2}`
    closure_span: Span,
    /// Block ID of the closure body for AST-based span extraction
    block_id: BlockId,
    param_name: String,
    param_var_id: VarId,
}

struct ClosureParameter {
    name: String,
    var_id: VarId,
}

/// Check if an expression is a stored closure (variable or cell path
/// reference). We skip these because the user explicitly stored the closure for
/// reuse.
const fn is_stored_closure(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::Var(_) | Expr::FullCellPath(_))
}

/// Extract the single parameter from a closure if it has exactly one parameter.
fn extract_closure_parameter(block_id: BlockId, context: &LintContext) -> Option<ClosureParameter> {
    let block = context.working_set.get_block(block_id);

    // Only process closures with exactly one required parameter
    let [param] = block.signature.required_positional.as_slice() else {
        return None;
    };

    let var_id = param.var_id?;
    let var = context.working_set.get_variable(var_id);
    let name = context.plain_text(var.declaration_span).to_string();

    Some(ClosureParameter { name, var_id })
}

/// Extract the body span from a closure block using AST structure.
///
/// The body span is computed from the block's pipeline elements, which
/// represent the actual executable content of the closure.
///
/// Returns `None` if the block has no pipeline elements.
fn extract_body_span_from_block(block_id: BlockId, context: &LintContext) -> Option<Span> {
    let block = context.working_set.get_block(block_id);
    let elements = block.all_elements();

    if elements.is_empty() {
        return None;
    }

    // Get the combined span of all pipeline elements
    let first_span = elements.first()?.expr.span;
    let last_span = elements.last()?.expr.span;

    Some(Span::new(first_span.start, last_span.end))
}

/// Check if a `where` or `filter` call uses a named parameter closure that
/// should use `$it` instead.
fn check_where_or_filter_call(
    call: &Call,
    context: &LintContext,
) -> Option<(Detection, RowConditionFixData)> {
    let command_name = call.get_call_name(context);
    if !matches!(command_name.as_str(), "where" | "filter") {
        return None;
    }

    let arg_expr = call.get_first_positional_arg()?;

    // Skip stored closures (variables containing closures)
    if is_stored_closure(arg_expr) {
        return None;
    }

    // Extract block ID from closure or row condition
    let block_id = match &arg_expr.expr {
        Expr::RowCondition(id) | Expr::Closure(id) => *id,
        _ => return None,
    };

    // Extract the single parameter
    let param = extract_closure_parameter(block_id, context)?;

    // Skip if already using `it`
    if param.name == "it" {
        return None;
    }

    // Verify the closure has explicit pipe syntax (not a block)
    let arg_text = context.plain_text(arg_expr.span);
    if !arg_text.contains(&format!("|{}|", param.name)) {
        return None;
    }

    let violation = Detection::from_global_span(
        "Use `$it` instead of closure parameter for more concise code",
        arg_expr.span,
    )
    .with_primary_label("closure with named parameter")
    .with_extra_label(format!("{command_name} command"), call.span());

    let fix_data = RowConditionFixData {
        closure_span: arg_expr.span,
        block_id,
        param_name: param.name,
        param_var_id: param.var_id,
    };

    Some((violation, fix_data))
}

fn check_expression(
    expr: &Expression,
    context: &LintContext,
) -> Vec<(Detection, RowConditionFixData)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    check_where_or_filter_call(call, context)
        .into_iter()
        .collect()
}

/// Sort and deduplicate spans by position.
fn deduplicate_spans(spans: &[Span]) -> Vec<Span> {
    let mut sorted = spans.to_vec();
    sorted.sort_by_key(|s| (s.start, s.end));
    sorted.dedup();
    sorted
}

/// Filter out nested spans, keeping only the innermost span for each position.
/// For example, if we have both `$x` and `$x.field` at the same start position,
/// keep only `$x` (the shorter span).
fn filter_nested_spans(spans: &[Span]) -> Vec<Span> {
    spans
        .iter()
        .filter(|span| {
            !spans
                .iter()
                .any(|other| other.start == span.start && other.end > span.end)
        })
        .copied()
        .collect()
}

/// Replace a parameter name with `it` in a variable reference.
///
/// Examples:
/// - `$x` → `$it`
/// - `$x.field` → `$it.field`
///
/// Assumes `var_text` starts with `$` followed by the parameter name.
fn replace_param_with_it(var_text: &str, param_name: &str) -> String {
    // Strip '$' then strip parameter name, keeping any suffix like '.field'
    let after_dollar = var_text
        .strip_prefix('$')
        .expect("variable must start with $");
    let suffix = after_dollar
        .strip_prefix(param_name)
        .expect("variable must start with parameter name");
    format!("$it{suffix}")
}

struct WhereClosureToIt;

impl DetectFix for WhereClosureToIt {
    type FixInput<'a> = RowConditionFixData;

    fn id(&self) -> &'static str {
        "where_closure_drop_parameter"
    }

    fn short_description(&self) -> &'static str {
        "You can drop the closure and its parameter in 'where' and 'filter'."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/where.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
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

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // Extract body span and variable usages from the block (AST-based)
        let block = context.working_set.get_block(fix_data.block_id);
        let body_span = extract_body_span_from_block(fix_data.block_id, context)?;
        let var_usage_spans =
            block.find_var_usage_spans(fix_data.param_var_id, context, |_, _, _| true);

        // Build the new body by replacing all parameter usages with $it
        let var_spans = deduplicate_spans(&var_usage_spans);
        let filtered_spans = filter_nested_spans(&var_spans);

        // Sort spans by position and filter to body range
        let mut sorted_spans: Vec<_> = filtered_spans
            .into_iter()
            .filter(|span| span.start >= body_span.start && span.end <= body_span.end)
            .collect();
        sorted_spans.sort_by_key(|s| s.start);

        // Build the replacement text by processing the body from left to right
        let mut result = String::new();
        let mut last_end = body_span.start;

        for span in &sorted_spans {
            // Add text before this variable
            if span.start > last_end {
                let before_span = Span::new(last_end, span.start);
                result.push_str(context.plain_text(before_span));
            }
            // Add the replacement for this variable
            let var_text = context.plain_text(*span);
            result.push_str(&replace_param_with_it(var_text, &fix_data.param_name));
            last_end = span.end;
        }

        // Add any remaining text after the last variable
        if last_end < body_span.end {
            let after_span = Span::new(last_end, body_span.end);
            result.push_str(context.plain_text(after_span));
        }

        // If no variables were replaced, just use the original body
        if sorted_spans.is_empty() {
            result = context.plain_text(body_span).to_string();
        }

        // Replace the entire closure with just the body (row condition syntax)
        let replacements = vec![Replacement::new(fix_data.closure_span, result)];

        Some(Fix::with_explanation(
            format!(
                "Replace closure parameter ${} with row condition using $it",
                fix_data.param_name
            ),
            replacements,
        ))
    }
}

pub static RULE: &dyn Rule = &WhereClosureToIt;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
