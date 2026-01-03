use std::cmp::Ordering;

use nu_protocol::{
    BlockId, Span,
    ast::{Call, Expr},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    span::FileSpan,
};

#[derive(Debug, Clone)]
pub struct CustomCommandDef {
    pub body: BlockId,
    pub name: String,
    pub name_span: Span,
    pub signature_span: Span,
    pub export_span: Option<Span>,
    pub signature: nu_protocol::Signature,
}

impl PartialEq for CustomCommandDef {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
    }
}

impl Eq for CustomCommandDef {}

impl PartialOrd for CustomCommandDef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CustomCommandDef {
    fn cmp(&self, other: &Self) -> Ordering {
        self.body.cmp(&other.body)
    }
}

impl CustomCommandDef {
    pub(crate) const fn new(
        body: BlockId,
        name: String,
        name_span: Span,
        signature_span: Span,
        export_span: Option<Span>,
        signature: nu_protocol::Signature,
    ) -> Self {
        Self {
            body,
            name,
            name_span,
            signature_span,
            export_span,
            signature,
        }
    }

    pub fn try_from_call(call: &Call, context: &LintContext) -> Option<Self> {
        let decl_name = call.get_call_name(context);

        let is_exported = match decl_name.as_str() {
            "export def" => true,
            "def" => false,
            _ => return None,
        };

        let name_arg = call.get_first_positional_arg()?;
        let name = match &name_arg.expr {
            Expr::String(s) | Expr::RawString(s) => s.clone(),
            _ => context.get_span_text(name_arg.span).to_string(),
        };

        let signature_expr = call.get_positional_arg(1)?;
        let signature_span = signature_expr.span;

        let body_expr = call.get_positional_arg(2)?;
        let block_id = body_expr.extract_block_id()?;

        let block = context.working_set.get_block(block_id);
        let signature = (*block.signature).clone();

        let export_span = is_exported.then(|| Span::new(call.head.start, call.head.start + 7));

        Some(Self::new(
            block_id,
            name,
            name_arg.span,
            signature_span,
            export_span,
            signature,
        ))
    }

    pub fn is_main(&self) -> bool {
        self.name == "main" || self.name.starts_with("main ")
    }

    pub const fn is_exported(&self) -> bool {
        self.export_span.is_some()
    }

    /// Get the declaration span as a `FileSpan` for use in detections
    /// This requires the `LintContext` to normalize the global span to
    /// file-relative positions
    pub const fn declaration_span(&self, context: &LintContext) -> FileSpan {
        context.normalize_span(self.name_span)
    }
}
