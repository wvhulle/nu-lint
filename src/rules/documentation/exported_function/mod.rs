use nu_protocol::ast::{Call, Expr};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

/// Check if there's a documentation comment before the given span
/// Since comments are not preserved in AST, we need to check source text
fn has_doc_comment_before(context: &LintContext, span: nu_protocol::Span) -> bool {
    let before_text = context.source_before_span(span);
    let lines: Vec<&str> = before_text.lines().collect();

    if lines.is_empty() {
        return false;
    }

    // Look for documentation comments, skipping over attributes and empty lines
    for line in lines.iter().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip attribute lines (like @example, @search-terms, etc.)
        if trimmed.starts_with('@') {
            continue;
        }

        // Check if it's a documentation comment (not a shebang ##)
        let is_comment = trimmed.starts_with('#') && !trimmed.starts_with("##");
        return is_comment;
    }

    false
}

fn check_exported_function(call: &Call, context: &LintContext) -> Option<Detection> {
    let func_def = call.custom_command_def(context)?;

    if !func_def.is_exported() {
        return None;
    }

    let has_docs = has_doc_comment_before(context, call.head);

    if has_docs {
        None
    } else {
        Some(
            Detection::from_global_span(
                format!(
                    "Exported function '{}' lacks documentation comment",
                    func_def.name
                ),
                call.head,
            )
            .with_primary_label("missing doc comment")
            .with_extra_label("exported function", func_def.name_span),
        )
    }
}

struct AddDocCommentExportedFn;

impl DetectFix for AddDocCommentExportedFn {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "add_doc_comment_exported_fn"
    }

    fn short_description(&self) -> &'static str {
        "Exported functions should have documentation comments"
    }
    fn long_description(&self) -> Option<&'static str> {
        Some("Add a documentation comment (starting with #) above the export.")
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#documenting-your-command")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, ctx| {
            if let Expr::Call(call) = &expr.expr {
                check_exported_function(call, ctx).into_iter().collect()
            } else {
                vec![]
            }
        }))
    }
}

pub static RULE: &dyn Rule = &AddDocCommentExportedFn;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
