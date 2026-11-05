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

    fn is_inside_any(&self, container_spans: &[Span]) -> bool {
        container_spans
            .iter()
            .any(|s| s.start <= self.start && s.end >= self.end)
    }
}
