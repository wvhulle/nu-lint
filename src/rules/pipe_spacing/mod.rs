use nu_protocol::Span;

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// AST visitor that checks for pipe spacing issues
struct PipeSpacingVisitor<'a> {
    source: &'a str,
    violations: Vec<RuleViolation>,
}

impl<'a> PipeSpacingVisitor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            violations: Vec::new(),
        }
    }

    /// Check spacing around a pipe between two elements (optimized)
    fn check_pipe_spacing(&mut self, prev_span: Span, curr_span: Span) {
        let start = prev_span.end;
        let end = curr_span.start;

        if start >= end || end > self.source.len() {
            return;
        }

        // Check the actual text to see if it's already correctly formatted
        let between = &self.source[start..end];

        // If it's exactly " | ", it's correct
        if between == " | " {
            return;
        }

        // Check if it's a multi-line pipe using minimal line counting
        let prev_line = self.get_line_number_optimized(prev_span.end);
        let curr_line = self.get_line_number_optimized(curr_span.start);

        if prev_line != curr_line {
            // Multi-line pipe is fine - skip checking
            return;
        }

        // Skip if this region contains a comment (prevents false positives)
        if between.contains('#') || between.contains("//") {
            return;
        }

        // Same line: check for proper spacing
        if let Some(pipe_pos) = between.find('|') {
            // Check if this is inside closure parameters like |x|
            if Self::is_closure_parameter(between, pipe_pos) {
                return;
            }

            let before_pipe = &between[..pipe_pos];
            let after_pipe = &between[pipe_pos + 1..];

            let has_proper_space_before = before_pipe == " ";
            let has_proper_space_after =
                after_pipe.starts_with(' ') && !after_pipe.starts_with("  ");

            if has_proper_space_before && has_proper_space_after {
                return;
            }

            let message = Self::get_pipe_spacing_message(has_proper_space_before, has_proper_space_after, before_pipe);

            let violation_start = start + pipe_pos.saturating_sub(1);
            let violation_end = (start + pipe_pos + 2).min(end);
            let violation_span = Span::new(violation_start, violation_end);

            let fix_start = start;
            let fix_end = end;
            let fix_span = Span::new(fix_start, fix_end);

            let fix = Fix::new_static(
                "Fix pipe spacing to ' | '",
                vec![Replacement::new_static(fix_span, " | ")],
            );

            self.violations.push(
                RuleViolation::new_dynamic("pipe_spacing", message.to_string(), violation_span)
                    .with_suggestion_static("Use ' | ' with single spaces")
                    .with_fix(fix),
            );
        }
    }

    fn get_pipe_spacing_message(has_proper_space_before: bool, has_proper_space_after: bool, before_pipe: &str) -> &'static str {
        if !has_proper_space_before && !has_proper_space_after {
            "Pipe should have exactly one space before and after"
        } else if !has_proper_space_before {
            if before_pipe.is_empty() {
                "Pipe should have space before |"
            } else {
                "Pipe should have exactly one space before |"
            }
        } else {
            "Pipe should have space after |"
        }
    }

    fn is_closure_parameter(text: &str, pipe_pos: usize) -> bool {
        let before = &text[..pipe_pos];
        let after = &text[pipe_pos + 1..];
        before.contains('{') && after.contains('|')
    }

    fn get_line_number_optimized(&self, offset: usize) -> usize {
        // Only count newlines up to the offset, not the entire source
        let safe_offset = offset.min(self.source.len());
        self.source[..safe_offset]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
    }
}

fn check_pipeline_spacing(
    pipeline: &nu_protocol::ast::Pipeline,
    source: &str,
) -> Vec<RuleViolation> {
    let mut violations = Vec::new();
    let mut visitor = PipeSpacingVisitor::new(source);

    // Check spacing between consecutive pipeline elements
    for i in 1..pipeline.elements.len() {
        let prev = &pipeline.elements[i - 1];
        let curr = &pipeline.elements[i];
        visitor.check_pipe_spacing(prev.expr.span, curr.expr.span);
    }

    violations.extend(visitor.violations);
    violations
}

fn walk_block_for_pipelines(
    block: &nu_protocol::ast::Block,
    working_set: &nu_protocol::engine::StateWorkingSet,
    source: &str,
    violations: &mut Vec<RuleViolation>,
) {
    for pipeline in &block.pipelines {
        violations.extend(check_pipeline_spacing(pipeline, source));

        // Also check nested blocks
        for element in &pipeline.elements {
            walk_expr_for_pipelines(&element.expr, working_set, source, violations);
        }
    }
}

fn walk_expr_for_pipelines(
    expr: &nu_protocol::ast::Expression,
    working_set: &nu_protocol::engine::StateWorkingSet,
    source: &str,
    violations: &mut Vec<RuleViolation>,
) {
    match &expr.expr {
        nu_protocol::ast::Expr::Block(block_id)
        | nu_protocol::ast::Expr::Closure(block_id)
        | nu_protocol::ast::Expr::Subexpression(block_id)
        | nu_protocol::ast::Expr::RowCondition(block_id) => {
            let block = working_set.get_block(*block_id);
            walk_block_for_pipelines(block, working_set, source, violations);
        }
        nu_protocol::ast::Expr::Call(call) => {
            for arg in &call.arguments {
                if let Some(expr) = arg.expr() {
                    walk_expr_for_pipelines(expr, working_set, source, violations);
                }
            }
        }
        _ => {}
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();
    walk_block_for_pipelines(
        context.ast,
        context.working_set,
        context.source,
        &mut violations,
    );
    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "pipe_spacing",
        RuleCategory::Formatting,
        Severity::Warning,
        "Pipes should have exactly one space before and after when on the same line",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
