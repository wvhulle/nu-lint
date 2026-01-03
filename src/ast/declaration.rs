use std::cmp::Ordering;

use nu_protocol::{
    BlockId, Span,
    ast::{Call, Expr},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
};

#[derive(Debug, Clone)]
pub struct CustomCommandDef {
    pub body: BlockId,
    pub name: String,
    pub name_span: Span,
    pub export_span: Option<Span>,
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
        export_span: Option<Span>,
    ) -> Self {
        Self {
            body,
            name,
            name_span,
            export_span,
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

        let body_expr = call.get_positional_arg(2)?;
        let block_id = body_expr.extract_block_id()?;

        let export_span = is_exported.then(|| Span::new(call.head.start, call.head.start + 7));

        Some(Self::new(block_id, name, name_arg.span, export_span))
    }

    pub fn is_main(&self) -> bool {
        self.name == "main" || self.name.starts_with("main ")
    }

    pub const fn is_exported(&self) -> bool {
        self.export_span.is_some()
    }
}
