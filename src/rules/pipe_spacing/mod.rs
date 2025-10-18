use nu_protocol::Span;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{AstRule, RuleCategory, RuleMetadata},
    visitor::{AstVisitor, VisitContext},
};

#[derive(Default)]
pub struct PipeSpacing;

impl RuleMetadata for PipeSpacing {
    fn id(&self) -> &'static str {
        "pipe_spacing"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Formatting
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Pipes should have exactly one space before and after when on the same line"
    }
}

impl AstRule for PipeSpacing {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = PipeSpacingVisitor::new(self, context.source);
        context.walk_ast(&mut visitor);
        visitor.violations
    }
}

/// AST visitor that checks for pipe spacing issues
pub struct PipeSpacingVisitor<'a> {
    rule: &'a PipeSpacing,
    source: &'a str,
    violations: Vec<Violation>,
}

impl<'a> PipeSpacingVisitor<'a> {
    #[must_use]
    pub fn new(rule: &'a PipeSpacing, source: &'a str) -> Self {
        Self {
            rule,
            source,
            violations: Vec::new(),
        }
    }

    /// Check spacing around a pipe between two elements (optimized)
    fn check_pipe_spacing(&mut self, prev_span: Span, curr_span: Span, _context: &VisitContext) {
        use crate::lint::{Fix, Replacement};

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

            // Fixed logic: Check for exactly one space before and after
            let has_proper_space_before = before_pipe == " ";
            let has_proper_space_after =
                after_pipe.starts_with(' ') && !after_pipe.starts_with("  ");

            if !has_proper_space_before || !has_proper_space_after {
                let message = if !has_proper_space_before && !has_proper_space_after {
                    "Pipe should have exactly one space before and after"
                } else if !has_proper_space_before {
                    if before_pipe.is_empty() {
                        "Pipe should have space before |"
                    } else {
                        "Pipe should have exactly one space before |"
                    }
                } else {
                    "Pipe should have space after |"
                };

                // Calculate the span for the violation (around the pipe)
                let violation_start = start + pipe_pos.saturating_sub(1);
                let violation_end = (start + pipe_pos + 2).min(end);
                let violation_span = Span::new(violation_start, violation_end);

                // Replace the entire pipe region with proper spacing
                let fix_start = start;
                let fix_end = end;
                let fix_span = Span::new(fix_start, fix_end);

                let fix = Some(Fix {
                    description: "Fix pipe spacing to ' | '".to_string(),
                    replacements: vec![Replacement {
                        span: fix_span,
                        new_text: " | ".to_string(),
                    }],
                });

                self.violations.push(Violation {
                    rule_id: self.rule.id().to_string(),
                    severity: self.rule.severity(),
                    message: message.to_string(),
                    span: violation_span,
                    suggestion: Some("Use ' | ' with single spaces".to_string()),
                    fix,
                    file: None,
                });
            }
        }
    }

    /// Check if a pipe at the given position is part of closure parameters
    fn is_closure_parameter(text: &str, pipe_pos: usize) -> bool {
        // Look for pattern like {|...|
        let before = &text[..pipe_pos];
        let after = &text[pipe_pos + 1..];

        // Check if we're between { and another |
        before.contains('{') && after.contains('|')
    }

    /// Get line number for a byte offset (optimized for performance)
    fn get_line_number_optimized(&self, offset: usize) -> usize {
        // Only count newlines up to the offset, not the entire source
        let safe_offset = offset.min(self.source.len());
        self.source[..safe_offset]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
    }
}

impl AstVisitor for PipeSpacingVisitor<'_> {
    fn visit_pipeline(&mut self, pipeline: &nu_protocol::ast::Pipeline, context: &VisitContext) {
        // Check spacing between consecutive pipeline elements
        for i in 1..pipeline.elements.len() {
            let prev = &pipeline.elements[i - 1];
            let curr = &pipeline.elements[i];

            self.check_pipe_spacing(prev.expr.span, curr.expr.span, context);
        }

        // Continue walking the tree
        for element in &pipeline.elements {
            self.visit_expression(&element.expr, context);
        }
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
