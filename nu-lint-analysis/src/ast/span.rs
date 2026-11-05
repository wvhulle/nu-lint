use std::collections::BTreeSet;

use nu_protocol::Span;

use crate::{
    ast::{block::BlockExt, declaration::CustomCommandDef},
    context::LintContext,
};

pub trait SpanExt {
    #[must_use]
    /// Finds function containing this span. Example: statement span inside `def
    /// process [] { ... }`
    fn find_containing_function<'a>(
        &self,
        functions: &'a BTreeSet<CustomCommandDef>,
        context: &LintContext,
    ) -> Option<&'a CustomCommandDef>;
    #[must_use]
    /// Check if there's a documentation comment on the same line as this span
    /// This is used for inline parameter documentation: `param # Description`
    fn has_inline_doc_comment(&self, context: &LintContext) -> bool;
    #[must_use]
    /// Check if this span is contained within any of the provided container
    /// spans
    fn is_inside_any(&self, container_spans: &[Span]) -> bool;
}

impl SpanExt for Span {
    fn find_containing_function<'a>(
        &self,
        functions: &'a BTreeSet<CustomCommandDef>,
        context: &LintContext,
    ) -> Option<&'a CustomCommandDef> {
        functions
            .iter()
            .filter(|def| context.working_set.get_block(def.body).contains_span(*self))
            .min_by_key(|def| {
                let block = context.working_set.get_block(def.body);
                block.span.map_or(usize::MAX, |s| s.end - s.start)
            })
    }

    fn has_inline_doc_comment(&self, context: &LintContext) -> bool {
        // Get the source text after this span
        let after_text = context.source_after_span(*self);

        // Find the end of the line (either newline or end of file)
        let line_end = after_text.find('\n').unwrap_or(after_text.len());

        let rest_of_line = &after_text[..line_end];

        // Check if the rest of the line contains a documentation comment
        // For typed parameters like "count: int # Description", the span only covers
        // "count" so we need to check if " # " appears anywhere on the rest of
        // the line
        rest_of_line.contains(" # ")
    }

    fn is_inside_any(&self, container_spans: &[Span]) -> bool {
        container_spans
            .iter()
            .any(|s| s.start <= self.start && s.end >= self.end)
    }
}
