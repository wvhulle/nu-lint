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
    /// Span of the entire definition (from `def`/`export def` to closing `}`)
    pub definition_span: Span,
    /// Span of the body expression (the block argument)
    pub body_expr_span: Span,
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
            _ => context.expr_text(name_arg).to_string(),
        };

        let signature_expr = call.get_positional_arg(1)?;
        let signature_span = signature_expr.span;

        let body_expr = call.get_positional_arg(2)?;
        let block_id = body_expr.extract_block_id()?;

        let block = context.working_set.get_block(block_id);
        let signature = (*block.signature).clone();

        let export_span = is_exported.then(|| Span::new(call.head.start, call.head.start + 7));

        // The definition span is the span of the entire call expression
        let definition_span = call.span();

        Some(Self {
            body: block_id,
            name,
            name_span: name_arg.span,
            signature_span,
            export_span,
            signature,
            definition_span,
            body_expr_span: body_expr.span,
        })
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

#[cfg(test)]
mod tests {
    use crate::context::LintContext;

    #[test]
    fn test_custom_command_spans() {
        let code = r#"
def get_first [] {
    $in | first
}

def main [] {
    [1 2 3] | get_first
}
"#;
        LintContext::test_with_parsed_source(code, |context| {
            let commands = context.custom_commands();
            assert_eq!(commands.len(), 2);

            // Verify the get_first command has correct spans
            let get_first = commands.iter().find(|c| c.name == "get_first").unwrap();
            assert!(
                context
                    .span_text(get_first.definition_span)
                    .starts_with("def get_first")
            );
            assert!(
                context
                    .span_text(get_first.body_expr_span)
                    .contains("$in | first")
            );
        });
    }
}
