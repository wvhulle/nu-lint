use nu_protocol::{
    BlockId, Span, Type,
    ast::{Block, Call},
};

use crate::{
    ast::{call::CallExt, syntax_shape::SyntaxShapeExt},
    context::LintContext,
};

pub mod missing_argument_type;
pub mod missing_in_type;
pub mod missing_output_type;
pub mod paths;

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

fn format_positional(
    name: &str,
    shape: &nu_protocol::SyntaxShape,
    optional: bool,
    rest: bool,
) -> String {
    use crate::ast::syntax_shape::SyntaxShapeExt;

    let prefix = if rest { "..." } else { "" };
    let suffix = if optional { "?" } else { "" };

    match shape {
        nu_protocol::SyntaxShape::Any => format!("{prefix}{name}{suffix}"),
        _ => format!("{prefix}{name}{suffix}: {}", shape.to_type_string()),
    }
}

pub fn extract_parameters_text(signature: &nu_protocol::Signature) -> String {
    let required = signature
        .required_positional
        .iter()
        .map(|param| format_positional(&param.name, &param.shape, false, false));

    let optional = signature
        .optional_positional
        .iter()
        .map(|param| format_positional(&param.name, &param.shape, true, false));

    let rest = signature
        .rest_positional
        .iter()
        .map(|rest| format_positional(&rest.name, &rest.shape, false, true));

    let flags = signature
        .named
        .iter()
        .filter(|flag| flag.long != "help")
        .map(|flag| match (&flag.short, &flag.arg) {
            (Some(short), Some(arg_shape)) => {
                format!(
                    "--{} (-{}): {}",
                    flag.long,
                    short,
                    arg_shape.to_type_string()
                )
            }
            (Some(short), None) => format!("--{} (-{})", flag.long, short),
            (None, Some(arg_shape)) => {
                format!("--{}: {}", flag.long, arg_shape.to_type_string())
            }
            (None, None) => format!("--{}", flag.long),
        });

    required
        .chain(optional)
        .chain(rest)
        .chain(flags)
        .collect::<Vec<_>>()
        .join(", ")
}

pub struct ParsedSignature {
    pub input_type: Option<Type>,
    pub output_type: Option<Type>,
}

pub fn parse_signature_types(sig_text: &str) -> ParsedSignature {
    let type_annotation = if let Some(colon_pos) = sig_text.rfind("]:") {
        &sig_text[colon_pos + 2..].trim()
    } else {
        return ParsedSignature {
            input_type: None,
            output_type: None,
        };
    };

    type_annotation.find("->").map_or(
        ParsedSignature {
            input_type: None,
            output_type: None,
        },
        |arrow_pos| {
            let input_str = type_annotation[..arrow_pos].trim();
            let output_str = type_annotation[arrow_pos + 2..].trim();

            let input_type = parse_type_string(input_str);
            let output_type = parse_type_string(output_str);

            ParsedSignature {
                input_type,
                output_type,
            }
        },
    )
}

fn parse_type_string(type_str: &str) -> Option<Type> {
    match type_str {
        "any" => Some(Type::Any),
        "nothing" => Some(Type::Nothing),
        "int" => Some(Type::Int),
        "float" => Some(Type::Float),
        "bool" => Some(Type::Bool),
        "string" => Some(Type::String),
        "record" => Some(Type::Record(vec![].into())),
        "table" => Some(Type::Table(vec![].into())),
        "list" => Some(Type::List(Box::new(Type::Any))),
        s if s.starts_with("list<") && s.ends_with('>') => {
            let inner = &s[5..s.len() - 1];
            parse_type_string(inner).map(|t| Type::List(Box::new(t)))
        }
        _ => Some(Type::Any),
    }
}

pub struct FixData {
    pub sig_span: Span,
    pub body_block_id: BlockId,
}
