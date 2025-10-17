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
        RuleCategory::Style
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

    fn create_visitor<'a>(&'a self, context: &'a LintContext<'a>) -> Box<dyn AstVisitor + 'a> {
        Box::new(PipeSpacingVisitor::new(self, context.source))
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

    /// Check spacing around a pipe between two elements
    fn check_pipe_spacing(&mut self, prev_span: Span, curr_span: Span, _context: &VisitContext) {
        // Get the text between the two spans (this contains the pipe)
        let start = prev_span.end;
        let end = curr_span.start;

        if start >= end || end > self.source.len() {
            return;
        }

        let between = &self.source[start..end];

        // Skip if this region contains a comment (Nushell # or Rust-style //)
        // This prevents false positives when comments mention pipes like |x|
        if between.contains('#') || between.contains("//") {
            return;
        }

        // Check if this is a multi-line pipe (idiomatic)
        let prev_line = Self::get_line_number(self.source, prev_span.end);
        let curr_line = Self::get_line_number(self.source, curr_span.start);

        if prev_line != curr_line {
            // Multi-line pipe is fine - skip checking
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

            let has_space_before = before_pipe.ends_with(' ') && !before_pipe.ends_with("  ");
            let has_space_after = after_pipe.starts_with(' ') && !after_pipe.starts_with("  ");

            if !has_space_before || !has_space_after {
                let message = if !has_space_before && !has_space_after {
                    "Pipe should have exactly one space before and after"
                } else if !has_space_before {
                    "Pipe should have space before |"
                } else {
                    "Pipe should have space after |"
                };

                // Calculate the span for the violation (around the pipe)
                let violation_start = start + pipe_pos.saturating_sub(1);
                let violation_end = (start + pipe_pos + 2).min(end);
                let violation_span = Span::new(violation_start, violation_end);

                self.violations.push(Violation {
                    rule_id: self.rule.id().to_string(),
                    severity: self.rule.severity(),
                    message: message.to_string(),
                    span: violation_span,
                    suggestion: Some("Use ' | ' with single spaces".to_string()),
                    fix: None,
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

    /// Get line number for a byte offset
    fn get_line_number(source: &str, offset: usize) -> usize {
        source[..offset].bytes().filter(|&b| b == b'\n').count()
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
