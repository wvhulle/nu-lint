use std::collections::HashMap;

use nu_protocol::{BlockId, Span};

use crate::{ast::block::BlockExt, context::LintContext};

pub trait SpanExt {
    #[must_use]
    /// Returns source text for this span. Example: span of `$x + 1` returns "$x
    /// + 1"
    fn text<'a>(&self, context: &'a LintContext) -> &'a str;
    #[must_use]
    /// Finds function containing this span. Example: statement span inside `def
    /// process [] { ... }`
    fn find_containing_function(
        &self,
        functions: &HashMap<BlockId, String>,
        context: &LintContext,
    ) -> Option<String>;
    #[must_use]
    /// Finds the span of a substring within this span. Example: finding
    /// parameter name within signature span
    fn find_substring_span(&self, substring: &str, context: &LintContext) -> Span;
    #[must_use]
    /// Check if there's a documentation comment on the same line as this span
    /// This is used for inline parameter documentation: `param # Description`
    fn has_inline_doc_comment(&self, context: &LintContext) -> bool;
}

impl SpanExt for Span {
    fn text<'a>(&self, context: &'a LintContext) -> &'a str {
        &context.source[self.start..self.end]
    }

    fn find_containing_function(
        &self,
        functions: &HashMap<BlockId, String>,
        context: &LintContext,
    ) -> Option<String> {
        functions
            .iter()
            .filter(|(block_id, _)| {
                context
                    .working_set
                    .get_block(**block_id)
                    .contains_span(*self)
            })
            .min_by_key(|(block_id, _)| {
                let block = context.working_set.get_block(**block_id);
                block.span.map_or(usize::MAX, |s| s.end - s.start)
            })
            .map(|(_, name)| name.clone())
    }

    fn find_substring_span(&self, substring: &str, context: &LintContext) -> Span {
        self.text(context)
            .as_bytes()
            .windows(substring.len())
            .position(|window| window == substring.as_bytes())
            .map_or(*self, |offset| {
                Self::new(self.start + offset, self.start + offset + substring.len())
            })
    }

    fn has_inline_doc_comment(&self, context: &LintContext) -> bool {
        let line_start = context.source[..self.start]
            .rfind('\n')
            .map_or(0, |pos| pos + 1);

        let line_end = context.source[self.end..]
            .find('\n')
            .map_or(context.source.len(), |pos| self.end + pos);

        let line_text = &context.source[line_start..line_end];

        line_text.contains(" # ")
    }
}
