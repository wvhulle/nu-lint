use nu_protocol::Span;

/// Fix data for external command alternatives
pub struct ExternalCmdFixData<'a> {
    pub arg_strings: Vec<&'a str>,
    pub expr_span: Span,
}
