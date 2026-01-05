use nu_protocol::{
    BlockId, Flag, PositionalArg, Span, SyntaxShape, Type,
    ast::{Block, Call},
};

use crate::{ast::call::CallExt, context::LintContext};

pub mod missing_argument_type;
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

/// Format a positional parameter (required, optional, or rest).
fn format_positional(p: &PositionalArg, optional: bool, rest: bool) -> String {
    let prefix = if rest { "..." } else { "" };
    let suffix = if optional { "?" } else { "" };
    match &p.shape {
        SyntaxShape::Any => format!("{prefix}{}{suffix}", p.name),
        shape => format!("{prefix}{}{suffix}: {shape}", p.name),
    }
}

/// Format a flag/named parameter.
fn format_flag(f: &Flag) -> String {
    let short = f.short.map(|s| format!(" (-{s})")).unwrap_or_default();
    let arg_type = f.arg.as_ref().map(|s| format!(": {s}")).unwrap_or_default();
    format!("--{}{short}{arg_type}", f.long)
}

/// Extract parameters from a signature as Nu source text.
pub fn extract_parameters_text(signature: &nu_protocol::Signature) -> String {
    signature
        .required_positional
        .iter()
        .map(|p| format_positional(p, false, false))
        .chain(
            signature
                .optional_positional
                .iter()
                .map(|p| format_positional(p, true, false)),
        )
        .chain(
            signature
                .rest_positional
                .iter()
                .map(|p| format_positional(p, false, true)),
        )
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
