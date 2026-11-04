use nu_lint_analysis::{
    ast::{call::CallExt, declaration::parameter_list_span},
    context::LintContext,
};
use nu_protocol::{
    BlockId, Span, SyntaxShape, Type,
    ast::{Block, Call},
};

pub mod add_type_hints_arguments;
pub mod missing_in_type;
pub mod missing_output_type;

#[must_use]
pub fn find_return_span(block: &Block) -> Option<Span> {
    block
        .pipelines
        .last()
        .and_then(|p| p.elements.last())
        .map(|e| e.expr.span)
}

#[must_use]
pub fn get_input_type(signature: &nu_protocol::Signature) -> Option<Type> {
    signature
        .input_output_types
        .first()
        .map(|(input, _)| input.clone())
}

#[must_use]
pub fn get_output_type(signature: &nu_protocol::Signature) -> Option<Type> {
    signature
        .input_output_types
        .first()
        .map(|(_, output)| output.clone())
}

#[must_use]
pub fn find_signature_span(call: &Call, _ctx: &LintContext) -> Option<Span> {
    let sig_arg = call.get_positional_arg(1)?;
    Some(sig_arg.span)
}

#[must_use]
pub fn parameter_list_text<'a>(signature_span: Span, context: &'a LintContext) -> Option<&'a str> {
    parameter_list_span(signature_span, context).map(|list_span| context.span_text(list_span))
}

/// Format a rest/variadic parameter with a custom shape (for list<T> -> T
/// conversion)
#[must_use]
pub fn format_rest_with_shape(name: &str, shape: &SyntaxShape) -> String {
    match shape {
        SyntaxShape::Any | SyntaxShape::ExternalArgument => format!("...{name}"),
        s => format!("...{name}: {s}"),
    }
}

pub struct FixData {
    pub sig_span: Span,
    pub body_block_id: BlockId,
}
