use std::cmp::Ordering;

use nu_parser::TokenContents;
use nu_protocol::{
    BlockId, PositionalArg, Span, SyntaxShape, Type,
    ast::{Call, Expr},
};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    span::FileSpan,
};

fn is_rest_type_annotated(rest: &PositionalArg, context: &LintContext) -> bool {
    rest.var_id
        .is_some_and(|var_id| context.working_set.get_variable(var_id).ty != Type::Any)
}

fn lex_signature_tokens(signature_span: Span, context: &LintContext) -> Vec<nu_parser::Token> {
    nu_parser::lex(
        context.working_set.get_span_contents(signature_span),
        signature_span.start,
        &[],
        b":",
        true,
    )
    .0
}

#[must_use]
pub fn parameter_list_span(signature_span: Span, context: &LintContext) -> Option<Span> {
    let parameter_list = lex_signature_tokens(signature_span, context).first()?.span;
    let contents = context.working_set.get_span_contents(parameter_list);
    (contents.first() == Some(&b'[') && contents.last() == Some(&b']')).then_some(parameter_list)
}

fn parameter_list_interior(signature_span: Span, context: &LintContext) -> Option<Span> {
    let parameter_list = parameter_list_span(signature_span, context)?;
    Some(Span::new(parameter_list.start + 1, parameter_list.end - 1))
}

fn lex_parameter_tokens(interior: Span, context: &LintContext) -> Vec<nu_parser::Token> {
    nu_parser::lex_signature(
        context.working_set.get_span_contents(interior),
        interior.start,
        b"\n\r",
        b":=,",
        false,
    )
    .0
}

fn token_has_text(
    token: Option<&nu_parser::Token>,
    expected: &[u8],
    context: &LintContext,
) -> bool {
    token.is_some_and(|present_token| {
        context.working_set.get_span_contents(present_token.span) == expected
    })
}

fn find_parameter_token(tokens: &[nu_parser::Token], name_span: Span) -> Option<usize> {
    tokens
        .iter()
        .position(|token| token.span.start <= name_span.start && name_span.start < token.span.end)
}

fn skip_short_flag_token(
    tokens: &[nu_parser::Token],
    token_index: usize,
    context: &LintContext,
) -> usize {
    let is_short_flag = tokens.get(token_index).is_some_and(|token| {
        context
            .working_set
            .get_span_contents(token.span)
            .starts_with(b"(-")
    });
    token_index + usize::from(is_short_flag)
}

#[must_use]
pub fn parameter_removal_span(
    name_span: Span,
    signature_span: Span,
    context: &LintContext,
) -> Option<Span> {
    let interior = parameter_list_interior(signature_span, context)?;
    let tokens = lex_parameter_tokens(interior, context);
    let name_index = find_parameter_token(&tokens, name_span)?;

    let mut next_index = skip_short_flag_token(&tokens, name_index + 1, context);
    for annotation_separator in [b":", b"="] {
        if token_has_text(tokens.get(next_index), annotation_separator, context) {
            next_index += 2;
        }
    }
    if token_has_text(tokens.get(next_index), b",", context) {
        next_index += 1;
    }
    while tokens
        .get(next_index)
        .is_some_and(|token| token.contents == TokenContents::Comment)
    {
        next_index += 1;
    }

    if let Some(next_token) = tokens.get(next_index) {
        return Some(Span::new(
            tokens[name_index].span.start,
            next_token.span.start,
        ));
    }

    let previous_token = tokens[..name_index]
        .iter()
        .rev()
        .find(|token| !token_has_text(Some(token), b",", context));
    let Some(previous_token) = previous_token else {
        return Some(interior);
    };
    Some(Span::new(previous_token.span.end, tokens.last()?.span.end))
}

#[must_use]
pub fn annotated_parameter_span(
    name_span: Span,
    signature_span: Span,
    context: &LintContext,
) -> Option<Span> {
    let interior = parameter_list_interior(signature_span, context)?;
    let tokens = lex_parameter_tokens(interior, context);
    let name_index = find_parameter_token(&tokens, name_span)?;
    if !token_has_text(tokens.get(name_index + 1), b":", context)
        || token_has_text(tokens.get(name_index + 3), b"=", context)
    {
        return None;
    }
    Some(Span::new(
        tokens[name_index].span.start,
        tokens.get(name_index + 2)?.span.end,
    ))
}

#[must_use]
pub fn param_type_annotation_span(
    param: &PositionalArg,
    signature_span: Span,
    context: &LintContext,
) -> Option<Span> {
    let name_span = context
        .working_set
        .get_variable(param.var_id?)
        .declaration_span;
    let lex_span = Span::new(
        name_span.start,
        parameter_list_interior(signature_span, context)?.end,
    );
    let (tokens, _) = nu_parser::lex_signature(
        context.working_set.get_span_contents(lex_span),
        lex_span.start,
        b"\n\r",
        b":=,",
        false,
    );
    match tokens.as_slice() {
        [name, colon, type_token, ..]
            if name.span == name_span
                && context.working_set.get_span_contents(colon.span) == b":" =>
        {
            Some(Span::new(colon.span.start, type_token.span.end))
        }
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct CustomCommandDef {
    pub body: BlockId,
    pub name: String,
    pub name_span: Span,
    pub signature_span: Span,
    pub export_span: Option<Span>,
    pub signature: nu_protocol::Signature,
    pub is_wrapped: bool,
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
    #[must_use]
    pub fn try_from_call(call: &Call, context: &LintContext) -> Option<Self> {
        let is_exported = match call.builtin_name(context)? {
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
        let Expr::Signature(parsed_signature) = &signature_expr.expr else {
            return None;
        };

        let body_expr = call.get_positional_arg(2)?;
        let block_id = body_expr.extract_block_id()?;

        let is_wrapped = call.has_named_flag("wrapped");
        let mut signature = (**parsed_signature).clone();
        signature.name.clone_from(&name);
        signature.allows_unknown_args = is_wrapped;
        if is_wrapped
            && let Some(rest) = &mut signature.rest_positional
            && !is_rest_type_annotated(rest, context)
        {
            rest.shape = SyntaxShape::ExternalArgument;
        }

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
            is_wrapped,
            definition_span,
            body_expr_span: body_expr.span,
        })
    }

    #[must_use]
    pub fn is_main(&self) -> bool {
        self.name == "main" || self.name.starts_with("main ")
    }

    #[must_use]
    pub const fn is_exported(&self) -> bool {
        self.export_span.is_some()
    }

    /// Get the declaration span as a `FileSpan` for use in detections
    /// This requires the `LintContext` to normalize the global span to
    /// file-relative positions
    #[must_use]
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
