use std::collections::HashMap;

use nu_protocol::{
    Span, VarId,
    ast::{Block, Call, Expr, Expression, Pipeline},
};

use super::{
    extract_delimiter_from_split_call, extract_index_from_call, is_indexed_access_call,
    is_split_row_call,
};
use crate::{
    Fix, LintLevel, Replacement,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

#[derive(Debug, Clone)]
struct SplitVariable {
    var_id: VarId,
    split_span: Span,
    input_span: Span,
    delimiter: Option<String>,
    has_filter: bool,
}

struct SplitVariableTracker {
    active_splits: HashMap<VarId, SplitVariable>,
}

impl SplitVariableTracker {
    fn new() -> Self {
        Self {
            active_splits: HashMap::new(),
        }
    }

    fn register_split(&mut self, split_var: SplitVariable) {
        self.active_splits.insert(split_var.var_id, split_var);
    }

    fn get_split(&self, var_id: VarId) -> Option<&SplitVariable> {
        self.active_splits.get(&var_id)
    }

    fn mark_filtered(&mut self, var_id: VarId) {
        if let Some(split_var) = self.active_splits.get_mut(&var_id) {
            split_var.has_filter = true;
        }
    }

    fn consume_split(&mut self, var_id: VarId) {
        self.active_splits.remove(&var_id);
    }

    fn invalidate_reassigned_vars(&mut self, pipeline: &Pipeline, context: &LintContext) {
        if pipeline.elements.is_empty() {
            return;
        }

        let first_expr = &pipeline.elements[0].expr;
        let Expr::Call(call) = &first_expr.expr else {
            return;
        };

        if let Some((var_id, _var_name, _span)) = call.extract_variable_declaration(context) {
            self.active_splits.remove(&var_id);
        }
    }
}

pub struct WithDelimiter {
    /// Span covering the entire pattern from assignment to index access
    full_span: Span,
    /// Span of the input expression being split (e.g., "a:b:c" in the
    /// subexpression)
    input_span: Span,
    /// Delimiter extracted from the split call
    delimiter: String,
    /// Index being accessed
    index: usize,
}

#[derive(Clone)]
struct MatchInfo {
    split_info: SplitVariable,
    access_span: Span,
    index: usize,
}

/// Extract a variable ID from an expression, handling both direct variables and
/// `FullCellPath` wrappers
fn extract_var_id_from_expr(expr: &Expression) -> Option<VarId> {
    match &expr.expr {
        Expr::Var(var_id) => Some(*var_id),
        Expr::FullCellPath(cell_path) if cell_path.tail.is_empty() => {
            if let Expr::Var(var_id) = &cell_path.head.expr {
                Some(*var_id)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Unwrap a subexpression to get its inner pipeline, if it contains exactly one
/// pipeline
fn unwrap_single_pipeline<'a>(
    block_id: nu_protocol::BlockId,
    context: &'a LintContext,
) -> Option<&'a Pipeline> {
    let block = context.working_set.get_block(block_id);
    (block.pipelines.len() == 1).then(|| &block.pipelines[0])
}

/// Search a pipeline for a split row call, optionally followed by filter
/// operations Returns `(span, input_span, delimiter, has_filter)` if found
fn find_split_in_pipeline(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<(Span, Span, Option<String>, bool)> {
    let mut split_span: Option<Span> = None;
    let mut input_span: Option<Span> = None;
    let mut delimiter: Option<String> = None;
    let mut has_filter = false;

    for element in &pipeline.elements {
        let expr_to_check = match &element.expr.expr {
            Expr::Call(_) => &element.expr,
            Expr::FullCellPath(cell_path) if cell_path.tail.is_empty() => {
                // Handle wrapped subexpressions like ("a:b" | split row ":")
                if let Expr::Subexpression(block_id) = &cell_path.head.expr
                    && let Some(inner_pipeline) = unwrap_single_pipeline(*block_id, context)
                {
                    return find_split_in_pipeline(inner_pipeline, context);
                }
                &cell_path.head
            }
            _ => continue,
        };

        let Expr::Call(call) = &expr_to_check.expr else {
            continue;
        };

        if is_split_row_call(call, context) {
            split_span = Some(element.expr.span);
            delimiter = extract_delimiter_from_split_call(call, context);
            // Get the input expression (first positional arg to split row)
            if let Some(first_elem) = pipeline.elements.first() {
                input_span = Some(first_elem.expr.span);
            }
        } else if is_filter_operation(call, context) {
            has_filter = true;
        }
    }

    split_span.and_then(|span| input_span.map(|inp| (span, inp, delimiter, has_filter)))
}

fn is_filter_operation(call: &Call, context: &LintContext) -> bool {
    matches!(
        call.get_call_name(context).as_str(),
        "where" | "filter" | "skip" | "take"
    )
}

/// Detect if a pipeline contains a variable assignment with a split row
/// expression Example: let split = ("a:b:c" | split row ":")
fn detect_split_assignment(pipeline: &Pipeline, context: &LintContext) -> Option<SplitVariable> {
    let first_expr = &pipeline.elements.first()?.expr;
    let Expr::Call(call) = &first_expr.expr else {
        return None;
    };

    let (var_id, _var_name, _span) = call.extract_variable_declaration(context)?;
    let value_expr = call.get_positional_arg(1)?;

    let inner_pipeline = match &value_expr.expr {
        Expr::Subexpression(block_id) | Expr::Block(block_id) => {
            unwrap_single_pipeline(*block_id, context)?
        }
        _ => return None,
    };

    let (_split_span, input_span, delimiter, has_filter) =
        find_split_in_pipeline(inner_pipeline, context)?;

    Some(SplitVariable {
        var_id,
        split_span: value_expr.span,
        input_span,
        delimiter,
        has_filter,
    })
}

/// Detect if a pipeline applies filter operations to a tracked split variable
/// Example: $split | where $it != ""
fn detect_filter_on_split(
    pipeline: &Pipeline,
    context: &LintContext,
    tracker: &SplitVariableTracker,
) -> Option<VarId> {
    if pipeline.elements.len() < 2 {
        return None;
    }

    let var_id = extract_var_id_from_expr(&pipeline.elements[0].expr)?;
    tracker.get_split(var_id)?;

    let has_filter = pipeline.elements[1..].iter().any(|element| {
        matches!(&element.expr.expr, Expr::Call(call) if is_filter_operation(call, context))
    });

    has_filter.then_some(var_id)
}

/// Detect if a pipeline performs indexed access on a tracked split variable
/// Example: $split | get 0
fn detect_index_access(
    pipeline: &Pipeline,
    context: &LintContext,
    tracker: &SplitVariableTracker,
) -> Option<MatchInfo> {
    if pipeline.elements.len() < 2 {
        return None;
    }

    let var_id = extract_var_id_from_expr(&pipeline.elements[0].expr)?;
    let split_info = tracker.get_split(var_id)?;

    pipeline.elements[1..].iter().find_map(|element| {
        let Expr::Call(call) = &element.expr.expr else {
            return None;
        };

        if !is_indexed_access_call(call, context) {
            return None;
        }

        let index = extract_index_from_call(call, context)?;

        Some(MatchInfo {
            split_info: split_info.clone(),
            access_span: element.expr.span,
            index,
        })
    })
}

fn create_violation(match_info: &MatchInfo, _context: &LintContext) -> (Detection, WithDelimiter) {
    let full_span = Span::new(
        match_info.split_info.split_span.start,
        match_info.access_span.end,
    );

    let delimiter = match_info
        .split_info
        .delimiter
        .as_ref()
        .expect("create_violation should only be called when delimiter is available");

    let violation = Detection::from_global_span(
        "Extract field directly with 'parse' instead of storing split result",
        full_span,
    )
    .with_primary_label("intermediate variable can be eliminated")
    .with_extra_label("split stored here", match_info.split_info.split_span)
    .with_extra_label("accessed by index here", match_info.access_span);

    let fix_data = WithDelimiter {
        full_span,
        input_span: match_info.split_info.input_span,
        delimiter: delimiter.clone(),
        index: match_info.index,
    };

    (violation, fix_data)
}

fn check_block(
    block: &Block,
    context: &LintContext,
    violations: &mut Vec<(Detection, WithDelimiter)>,
) {
    let mut tracker = SplitVariableTracker::new();

    for pipeline in &block.pipelines {
        // Invalidate variables that are being reassigned
        tracker.invalidate_reassigned_vars(pipeline, context);

        if let Some(split_var) = detect_split_assignment(pipeline, context) {
            tracker.register_split(split_var);
        }

        if let Some(var_id) = detect_filter_on_split(pipeline, context, &tracker) {
            tracker.mark_filtered(var_id);
        }

        if let Some(match_info) = detect_index_access(pipeline, context, &tracker) {
            let var_id = match_info.split_info.var_id;
            violations.push(create_violation(&match_info, context));
            tracker.consume_split(var_id);
        }
    }
}

struct SplitRowGetMultistatement;

impl DetectFix for SplitRowGetMultistatement {
    type FixInput<'a> = WithDelimiter;

    fn id(&self) -> &'static str {
        "split_row_get_multistatement"
    }

    fn short_description(&self) -> &'static str {
        "Extract field directly with 'parse' instead of storing split result"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Storing 'split row' in a variable and later accessing by index spreads related logic \
             across statements. Use 'parse' to extract fields directly with named access in a \
             single expression.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/parse.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();

        check_block(context.ast, context, &mut violations);

        violations.extend(context.detect_with_fix_data(|expr, ctx| {
            let mut expr_violations = Vec::new();

            if let Expr::Closure(block_id) | Expr::Block(block_id) = &expr.expr {
                let block = ctx.working_set.get_block(*block_id);
                check_block(block, ctx, &mut expr_violations);
            }

            expr_violations
        }));

        violations
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let WithDelimiter {
            full_span,
            input_span,
            delimiter,
            index,
        } = fix_data;

        let input_text = context.span_text(*input_span);

        // Calculate the number of fields by counting delimiters in the input
        // This gives us the actual field count from the original split
        let num_fields = input_text.matches(delimiter.as_str()).count() + 1;

        // Generate parse pattern with the correct number of fields
        let (pattern, needs_regex) = super::generate_parse_pattern(delimiter, num_fields);
        let parse_cmd = if needs_regex {
            format!("parse --regex '{pattern}'")
        } else {
            format!("parse \"{pattern}\"")
        };

        // The replacement should access the specific field from row 0
        // e.g., "input" | parse "{field0}:{field1}:{field2}" | get 0.field{index}
        let replacement = format!("{input_text} | {parse_cmd} | get 0.field{index}");

        Some(Fix::with_explanation(
            format!(
                "Replace multi-statement 'split row | get' pattern with '{parse_cmd} | get \
                 0.field{index}'"
            ),
            vec![Replacement::new(*full_span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &SplitRowGetMultistatement;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
