use std::collections::HashMap;

use nu_protocol::{
    Span, VarId,
    ast::{Block, Call, Expr, Expression, Pipeline},
};

use super::{
    extract_delimiter_from_split_call, extract_index_from_call, generate_parse_replacement,
    is_indexed_access_call, is_split_row_call,
};
use crate::{
    Fix, LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

#[derive(Debug, Clone)]
struct SplitVariable {
    var_id: VarId,
    split_span: Span,
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

pub enum FixData {
    WithDelimiter {
        combined_span: Span,
        delimiter: String,
        index: usize,
        _had_filter: bool,
    },
    NoFix {
        combined_span: Span,
    },
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
/// operations Returns `(span, delimiter, has_filter)` if found
fn find_split_in_pipeline(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<(Span, Option<String>, bool)> {
    let mut split_span: Option<Span> = None;
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
        } else if is_filter_operation(call, context) {
            has_filter = true;
        }
    }

    split_span.map(|span| (span, delimiter, has_filter))
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

    let (split_span, delimiter, has_filter) = find_split_in_pipeline(inner_pipeline, context)?;

    Some(SplitVariable {
        var_id,
        split_span,
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

fn create_violation(match_info: MatchInfo, _context: &LintContext) -> (Detection, FixData) {
    let combined_span = Span::new(
        match_info.split_info.split_span.start,
        match_info.access_span.end,
    );

    let help_message = match &match_info.split_info.delimiter {
        Some(delim) if !delim.is_empty() => {
            let replacement = generate_parse_replacement(delim, &[match_info.index]);
            format!(
                "Use '{replacement}' for structured text extraction. Access fields by name (e.g., \
                 $result.field{}) instead of index.",
                match_info.index
            )
        }
        _ => "Use 'parse \"{field0} {field1}\"' for structured text extraction. For complex \
              delimiters containing regex special characters, use 'parse --regex' with named \
              capture groups like '(?P<field0>.*)delimiter(?P<field1>.*)'"
            .to_string(),
    };

    let violation = Detection::from_global_span(
        "Manual string splitting with indexed access - consider using 'parse'",
        combined_span,
    )
    .with_primary_label("split + index pattern across statements")
    .with_help(help_message);

    let fix_data = if let Some(delimiter) = match_info.split_info.delimiter {
        FixData::WithDelimiter {
            combined_span,
            delimiter,
            index: match_info.index,
            _had_filter: match_info.split_info.has_filter,
        }
    } else {
        FixData::NoFix { combined_span }
    };

    (violation, fix_data)
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<(Detection, FixData)>) {
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
            violations.push(create_violation(match_info, context));
            tracker.consume_split(var_id);
        }
    }
}

struct SplitFilterIndexMultistatement;

impl DetectFix for SplitFilterIndexMultistatement {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "split_filter_index_multistatement"
    }

    fn explanation(&self) -> &'static str {
        "Prefer 'parse' command over 'split row | filter | get' pattern across multiple statements"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/parse.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
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

    fn fix(&self, _context: &LintContext, _fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        None
    }
}

pub static RULE: &dyn Rule = &SplitFilterIndexMultistatement;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
