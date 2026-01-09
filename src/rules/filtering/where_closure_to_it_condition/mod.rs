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
    param_decl_span: Span,
    param_name: String,
    var_usage_spans: Vec<Span>,
}

struct ClosureParameter {
    name: String,
    name_span: Span,
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

    Some(ClosureParameter {
        name,
        name_span: var.declaration_span,
        var_id,
    })
}

/// Find the span of the closure parameter declaration including pipes.
///
/// Given a parameter name span like the `x` in `{|x| $x > 2}`, this finds the
/// full declaration span `|x|` by:
/// 1. Scanning backward to find the opening `|`
/// 2. Scanning forward to find the closing `|`
///
/// Uses character indices for UTF-8 safety rather than byte slicing.
fn find_param_decl_span(
    block_span: Span,
    param_name_span: Span,
    context: &LintContext,
) -> Option<Span> {
    // Ensure the parameter name is within the block
    if param_name_span.start < block_span.start || param_name_span.end > block_span.end {
        return None;
    }

    let block_text = context.plain_text(block_span);
    let param_start = param_name_span.start - block_span.start;
    let param_end = param_name_span.end - block_span.start;

    // Collect character indices for UTF-8-safe iteration
    let char_indices: Vec<_> = block_text.char_indices().collect();

    // Find opening pipe before parameter
    let opening_pipe_offset = char_indices
        .iter()
        .rev()
        .find_map(|&(offset, ch)| (offset < param_start && ch == '|').then_some(offset))?;

    // Find closing pipe after parameter
    let closing_pipe_offset = char_indices
        .iter()
        .find_map(|&(offset, ch)| (offset >= param_end && ch == '|').then_some(offset))?;

    Some(Span::new(
        block_span.start + opening_pipe_offset,
        block_span.start + closing_pipe_offset + 1, // +1 to include the pipe itself
    ))
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

    let block = context.working_set.get_block(block_id);
    let block_span = block.span?;

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

    // Find the full parameter declaration span including pipes
    let param_decl_span = find_param_decl_span(block_span, param.name_span, context)?;

    // Collect all variable usages
    let var_usage_spans = block.find_var_usage_spans(param.var_id, context, |_, _, _| true);

    let violation = Detection::from_global_span(
        "Use `$it` instead of closure parameter for more concise code",
        arg_expr.span,
    )
    .with_primary_label("closure with named parameter")
    .with_extra_label(format!("{command_name} command"), call.span());

    let fix_data = RowConditionFixData {
        param_decl_span,
        param_name: param.name,
        var_usage_spans,
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
        let mut replacements = vec![
            // Remove the parameter declaration (|param_name|)
            Replacement::new(fix_data.param_decl_span, String::new()),
        ];

        // Deduplicate and filter nested variable usage spans
        let var_spans = deduplicate_spans(&fix_data.var_usage_spans);
        let filtered_spans = filter_nested_spans(&var_spans);

        // Replace each variable usage
        for &span in &filtered_spans {
            let var_text = context.plain_text(span);
            let replacement = replace_param_with_it(var_text, &fix_data.param_name);
            replacements.push(Replacement::new(span, replacement));
        }

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
