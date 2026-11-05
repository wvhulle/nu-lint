use nu_parser::{Token, TokenContents};
use nu_protocol::{Span, VarId};

use crate::context::LintContext;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParameterTokenRole {
    Colon,
    Equals,
    Comma,
    ShortFlag,
    Comment,
    Word,
}

#[derive(Clone, Copy)]
struct ParameterToken {
    span: Span,
    role: ParameterTokenRole,
}

fn classify_parameter_token(token: &Token, context: &LintContext) -> ParameterToken {
    let token_bytes = context.working_set.get_span_contents(token.span);
    let role = match token_bytes {
        _ if token.contents == TokenContents::Comment => ParameterTokenRole::Comment,
        b":" => ParameterTokenRole::Colon,
        b"=" => ParameterTokenRole::Equals,
        b"," => ParameterTokenRole::Comma,
        _ if token_bytes.starts_with(b"(-") => ParameterTokenRole::ShortFlag,
        _ => ParameterTokenRole::Word,
    };
    ParameterToken {
        span: token.span,
        role,
    }
}

fn find_parameter_list(signature_span: Span, context: &LintContext) -> Option<Span> {
    let (signature_tokens, _) = nu_parser::lex(
        context.working_set.get_span_contents(signature_span),
        signature_span.start,
        &[],
        b":",
        true,
    );
    let parameter_list = signature_tokens.first()?.span;
    let list_bytes = context.working_set.get_span_contents(parameter_list);
    (list_bytes.first() == Some(&b'[') && list_bytes.last() == Some(&b']'))
        .then_some(parameter_list)
}

pub struct SignatureLayout {
    signature_span: Span,
    parameter_list: Span,
    parameter_tokens: Vec<ParameterToken>,
}

impl SignatureLayout {
    #[must_use]
    pub fn parse(signature_span: Span, context: &LintContext) -> Option<Self> {
        let parameter_list = find_parameter_list(signature_span, context)?;
        let interior = Self::interior_of(parameter_list);
        let (parameter_tokens, _) = nu_parser::lex_signature(
            context.working_set.get_span_contents(interior),
            interior.start,
            b"\n\r",
            b":=,",
            false,
        );
        Some(Self {
            signature_span,
            parameter_list,
            parameter_tokens: parameter_tokens
                .iter()
                .map(|token| classify_parameter_token(token, context))
                .collect(),
        })
    }

    fn interior_of(parameter_list: Span) -> Span {
        Span::new(parameter_list.start + 1, parameter_list.end - 1)
    }

    #[must_use]
    pub fn input_output_annotation_span(&self) -> Span {
        Span::new(self.parameter_list.end, self.signature_span.end)
    }

    fn parameter_index(&self, var_id: VarId, context: &LintContext) -> Option<usize> {
        let name_start = context
            .working_set
            .get_variable(var_id)
            .declaration_span
            .start;
        self.parameter_tokens
            .iter()
            .position(|token| token.span.start <= name_start && name_start < token.span.end)
    }

    fn has_role(&self, token_index: usize, role: ParameterTokenRole) -> bool {
        self.parameter_tokens
            .get(token_index)
            .is_some_and(|token| token.role == role)
    }

    fn after_short_flag(&self, name_index: usize) -> usize {
        name_index + 1 + usize::from(self.has_role(name_index + 1, ParameterTokenRole::ShortFlag))
    }

    #[must_use]
    pub fn annotation_span(&self, var_id: VarId, context: &LintContext) -> Option<Span> {
        let colon_index = self.after_short_flag(self.parameter_index(var_id, context)?);
        if !self.has_role(colon_index, ParameterTokenRole::Colon) {
            return None;
        }
        Some(Span::new(
            self.parameter_tokens[colon_index].span.start,
            self.parameter_tokens.get(colon_index + 1)?.span.end,
        ))
    }

    #[must_use]
    pub fn annotated_entry_span(&self, var_id: VarId, context: &LintContext) -> Option<Span> {
        let name_index = self.parameter_index(var_id, context)?;
        let annotation = self.annotation_span(var_id, context)?;
        let has_default_value = self.has_role(
            self.after_short_flag(name_index) + 2,
            ParameterTokenRole::Equals,
        );
        (!has_default_value)
            .then(|| Span::new(self.parameter_tokens[name_index].span.start, annotation.end))
    }

    #[must_use]
    pub fn removal_span(&self, var_id: VarId, context: &LintContext) -> Option<Span> {
        let name_index = self.parameter_index(var_id, context)?;
        let mut next_index = self.after_short_flag(name_index);
        for clause_separator in [ParameterTokenRole::Colon, ParameterTokenRole::Equals] {
            if self.has_role(next_index, clause_separator) {
                next_index += 2;
            }
        }
        if self.has_role(next_index, ParameterTokenRole::Comma) {
            next_index += 1;
        }
        while self.has_role(next_index, ParameterTokenRole::Comment) {
            next_index += 1;
        }

        if let Some(next_token) = self.parameter_tokens.get(next_index) {
            return Some(Span::new(
                self.parameter_tokens[name_index].span.start,
                next_token.span.start,
            ));
        }

        let previous_token = self.parameter_tokens[..name_index]
            .iter()
            .rev()
            .find(|token| token.role != ParameterTokenRole::Comma);
        let Some(previous_token) = previous_token else {
            return Some(Self::interior_of(self.parameter_list));
        };
        Some(Span::new(
            previous_token.span.end,
            self.parameter_tokens.last()?.span.end,
        ))
    }
}
