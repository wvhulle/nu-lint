use nu_protocol::{BlockId, Span};

use super::BlockExt;
use crate::context::LintContext;

pub trait SpanExt {
    #[must_use]
    fn text<'a>(&self, context: &'a LintContext) -> &'a str;
    #[must_use]
    fn find_containing_function(
        &self,
        functions: &std::collections::HashMap<BlockId, String>,
        context: &LintContext,
    ) -> Option<String>;
}

impl SpanExt for Span {
    fn text<'a>(&self, context: &'a LintContext) -> &'a str {
        &context.source[self.start..self.end]
    }

    fn find_containing_function(
        &self,
        functions: &std::collections::HashMap<BlockId, String>,
        context: &LintContext,
    ) -> Option<String> {
        functions
            .iter()
            .filter(|(block_id, _)| block_id.contains_span(*self, context))
            .min_by_key(|(block_id, _)| {
                let block = context.working_set.get_block(**block_id);
                block.span.map_or(usize::MAX, |s| s.end - s.start)
            })
            .map(|(_, name)| name.clone())
    }
}
