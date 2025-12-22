use nu_protocol::{
    Span,
    ast::{Block, Expr, Expression, Pipeline},
};

use crate::{
    LintLevel,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

/// AST visitor that checks for pipe spacing issues
struct PipeSpacingVisitor<'a> {
    context: &'a LintContext<'a>,
    violations: Vec<Violation>,
}

impl<'a> PipeSpacingVisitor<'a> {
    const fn new(context: &'a LintContext<'a>) -> Self {
        Self {
            context,
            violations: Vec::new(),
        }
    }

    /// Check spacing around a pipe between two elements (optimized)
    fn check_pipe_spacing(&mut self, prev_span: Span, curr_span: Span) {
        let between = self.context.source_between_span_ends(prev_span, curr_span);

        if between.is_empty() {
            return;
        }

        // If it's exactly " | ", it's correct
        if between == " | " {
            return;
        }

        // Check if it's a multi-line pipe using minimal line counting
        let file_start = prev_span.end.saturating_sub(self.context.file_offset());
        let file_end = curr_span.start.saturating_sub(self.context.file_offset());

        let prev_line = self.context.count_newlines_before(file_start);
        let curr_line = self.context.count_newlines_before(file_end);

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

            let message = Self::get_pipe_spacing_message(
                has_proper_space_before,
                has_proper_space_after,
                before_pipe,
            );

            // Use global spans (will be normalized later by the engine)
            let start = prev_span.end;
            let end = curr_span.start;
            let violation_start = start + pipe_pos.saturating_sub(1);
            let violation_end = (start + pipe_pos + 2).min(end);
            let violation_span = Span::new(violation_start, violation_end);

            let fix_span = Span::new(start, end);

            let fix = Fix::with_explanation(
                "Fix pipe spacing to ' | '",
                vec![Replacement::new(fix_span, " | ")],
            );

            self.violations.push(
                Violation::new(message.to_string(), violation_span)
                    .with_primary_label("spacing issue")
                    .with_help("Use ' | ' with single spaces")
                    .with_fix(fix),
            );
        }
    }

    const fn get_pipe_spacing_message(
        has_proper_space_before: bool,
        has_proper_space_after: bool,
        before_pipe: &str,
    ) -> &'static str {
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
}

fn check_pipeline_spacing(pipeline: &Pipeline, context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut visitor = PipeSpacingVisitor::new(context);

    // Check spacing between consecutive pipeline elements
    for i in 1..pipeline.elements.len() {
        let prev = &pipeline.elements[i - 1];
        let curr = &pipeline.elements[i];
        visitor.check_pipe_spacing(prev.expr.span, curr.expr.span);
    }

    violations.extend(visitor.violations);
    violations
}

fn walk_block_for_pipelines(block: &Block, context: &LintContext, violations: &mut Vec<Violation>) {
    for pipeline in &block.pipelines {
        violations.extend(check_pipeline_spacing(pipeline, context));

        // Also check nested blocks
        for element in &pipeline.elements {
            walk_expr_for_pipelines(&element.expr, context, violations);
        }
    }
}

fn walk_expr_for_pipelines(
    expr: &Expression,
    context: &LintContext,
    violations: &mut Vec<Violation>,
) {
    match &expr.expr {
        Expr::Block(block_id)
        | Expr::Closure(block_id)
        | Expr::Subexpression(block_id)
        | Expr::RowCondition(block_id) => {
            let block = context.working_set.get_block(*block_id);
            walk_block_for_pipelines(block, context, violations);
        }
        Expr::Call(call) => {
            for arg in &call.arguments {
                if let Some(expr) = arg.expr() {
                    walk_expr_for_pipelines(expr, context, violations);
                }
            }
        }
        _ => {}
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();
    walk_block_for_pipelines(context.ast, context, &mut violations);
    violations
}

pub const fn rule() -> Rule {
    Rule::new(
        "pipe_spacing",
        "Pipes should have exactly one space before and after when on the same line",
        check,
        LintLevel::Warning,
    )
    .with_doc_url("https://www.nushell.sh/book/style_guide.html#basic")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
