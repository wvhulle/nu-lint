use std::ops::ControlFlow;

use nu_lint_analysis::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::{
    Span,
    ast::{Call, Expr},
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

        // Check if it's a documentation comment (not a shebang ##)
        let is_comment = trimmed.starts_with('#') && !trimmed.starts_with("##");
        return is_comment;
    }

    false
}

fn check_exported_function(
    call: &Call,
    documented_span: Span,
    context: &LintContext,
) -> Option<Detection> {
    let func_def = call.custom_command_def(context)?;

    if !func_def.is_exported() {
        return None;
    }

    let has_docs = has_doc_comment_before(context, documented_span);

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

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#documenting-your-command")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut detections = Vec::new();

        context.traverse_with_parent(|expr, parent| {
            let Expr::Call(call) = &expr.expr else {
                return ControlFlow::Continue(());
            };

            let documented_span = parent
                .filter(|p| matches!(p.expr, Expr::AttributeBlock(_)))
                .map_or(call.head, |p| p.span);

            detections.extend(check_exported_function(call, documented_span, context));
            ControlFlow::Continue(())
        });

        Self::no_fix(detections)
    }
}

pub static RULE: &dyn Rule = &AddDocCommentExportedFn;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
