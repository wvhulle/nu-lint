use nu_protocol::{
    BlockId, Span, SyntaxShape, Type,
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

pub fn extract_parameters_text(signature: &nu_protocol::Signature) -> String {
    let format_param = |name: &str, shape: &SyntaxShape, suffix: &str| match shape {
        SyntaxShape::Any => format!("{name}{suffix}"),
        _ => format!("{name}{suffix}: {shape}"), // Use upstream Display
    };

    let params = signature
        .required_positional
        .iter()
        .map(|p| format_param(&p.name, &p.shape, ""))
        .chain(
            signature
                .optional_positional
                .iter()
                .map(|p| format_param(&p.name, &p.shape, "?")),
        )
        .chain(
            signature
                .rest_positional
                .iter()
                .map(|p| format_param(&format!("...{}", p.name), &p.shape, "")),
        )
        .chain(
            signature
                .named
                .iter()
                .filter(|f| f.long != "help")
                .map(|f| {
                    let base = f.short.map_or_else(
                        || format!("--{}", f.long),
                        |s| format!("--{} (-{s})", f.long),
                    );
                    f.arg
                        .as_ref()
                        .map_or(base.clone(), |shape| format!("{base}: {shape}"))
                }),
        );

    params.collect::<Vec<_>>().join(", ")
}

pub struct FixData {
    pub sig_span: Span,
    pub body_block_id: BlockId,
}
