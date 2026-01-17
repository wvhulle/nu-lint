use nu_protocol::{
    BlockId, Flag, PositionalArg, Span, SyntaxShape, Type,
    ast::{Block, Call},
};

use crate::{ast::call::CallExt, context::LintContext};

pub mod add_type_hints_arguments;
pub mod missing_in_type;
pub mod missing_output_type;

pub fn find_return_span(block: &Block) -> Option<Span> {
    block
        .pipelines
        .last()
        .and_then(|p| p.elements.last())
        .map(|e| e.expr.span)
}

pub fn get_input_type(signature: &nu_protocol::Signature) -> Option<Type> {
    signature
        .input_output_types
        .first()
        .map(|(input, _)| input.clone())
}

pub fn get_output_type(signature: &nu_protocol::Signature) -> Option<Type> {
    signature
        .input_output_types
        .first()
        .map(|(_, output)| output.clone())
}

pub fn find_signature_span(call: &Call, _ctx: &LintContext) -> Option<Span> {
    let sig_arg = call.get_positional_arg(1)?;
    Some(sig_arg.span)
}

fn format_shape(name: &str, shape: &SyntaxShape) -> String {
    match shape {
        SyntaxShape::Any => name.to_string(),
        s => format!("{name}: {s}"),
    }
}

/// Format a required positional parameter: `name: type`
pub fn format_required(p: &PositionalArg) -> String {
    format_shape(&p.name, &p.shape)
}

/// Format an optional positional parameter: `name?: type`
pub fn format_optional(p: &PositionalArg) -> String {
    if matches!(p.shape, SyntaxShape::Any) {
        format!("{}?", p.name)
    } else {
        format!("{}?: {}", p.name, p.shape)
    }
}

/// Format a rest/variadic parameter: `...name: type`
pub fn format_rest(p: &PositionalArg) -> String {
    match &p.shape {
        SyntaxShape::Any => format!("...{}", p.name),
        s => format!("...{}: {s}", p.name),
    }
}

/// Format a rest/variadic parameter with a custom shape (for list<T> -> T
/// conversion)
pub fn format_rest_with_shape(name: &str, shape: &SyntaxShape) -> String {
    match shape {
        SyntaxShape::Any => format!("...{name}"),
        s => format!("...{name}: {s}"),
    }
}

/// Format a flag/named parameter.
pub fn format_flag(f: &Flag) -> String {
    let short = f.short.map(|s| format!(" (-{s})")).unwrap_or_default();
    let arg_type = f.arg.as_ref().map(|s| format!(": {s}")).unwrap_or_default();
    format!("--{}{short}{arg_type}", f.long)
}

/// Extract parameters from a signature as Nu source text.
pub fn extract_parameters_text(signature: &nu_protocol::Signature) -> String {
    signature
        .required_positional
        .iter()
        .map(format_required)
        .chain(signature.optional_positional.iter().map(format_optional))
        .chain(signature.rest_positional.iter().map(format_rest))
        .chain(
            signature
                .named
                .iter()
                .filter(|f| f.long != "help")
                .map(format_flag),
        )
        .collect::<Vec<_>>()
        .join(", ")
}

pub struct FixData {
    pub sig_span: Span,
    pub body_block_id: BlockId,
}
