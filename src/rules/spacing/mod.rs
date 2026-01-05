pub mod block_body_spacing;
pub mod closure_body_spacing;
pub mod closure_param_spacing;
pub mod no_trailing_spaces;
pub mod omit_list_commas;
pub mod pipe_spacing;
pub mod record_brace_spacing;
pub mod reflow_wide_pipelines;
pub mod wrap_long_lists;
pub mod wrap_records;

use nu_protocol::Span;

use crate::context::LintContext;

/// Checks if the source text for a block/closure contains explicit pipe
/// delimiters.
///
/// This distinguishes between:
/// - `{ body }` - block without pipe delimiters (returns false)
/// - `{|| body}` - closure with empty parameter pipes (returns true)
/// - `{|x| body}` - closure with parameters (returns true)
///
/// This is needed because Nu's AST doesn't distinguish between a block and a
/// closure with empty pipes - both have no parameters in the signature.
pub fn has_explicit_pipe_delimiters(context: &LintContext, span: Span) -> bool {
    let text = context.plain_text(span);
    let mut chars = text.chars();

    if chars.next() != Some('{') {
        return false;
    }

    // Look for opening pipe after optional whitespace
    // Pattern: `{` followed by optional whitespace then `|`
    chars.find(|c| !c.is_whitespace()) == Some('|')
}

/// Checks if the block has actual parameters (not just empty pipes `||`).
pub fn has_block_params(context: &LintContext, block_id: nu_protocol::BlockId) -> bool {
    let block = context.working_set.get_block(block_id);
    !block.signature.required_positional.is_empty()
        || !block.signature.optional_positional.is_empty()
        || block.signature.rest_positional.is_some()
}

/// Determines if a type is a record type.
pub const fn is_record_type(ty: &nu_protocol::Type) -> bool {
    matches!(ty, nu_protocol::Type::Record(_))
}
