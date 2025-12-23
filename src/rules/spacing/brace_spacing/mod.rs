use nu_protocol::{Span, ast::Expr};

use crate::{LintLevel, context::LintContext, rule::Rule, violation::Violation};
enum BraceType {
    ClosureWithParams,
    BlockWithoutParams,
    Record,
}

fn check_brace_spacing(
    context: &LintContext,
    span: Span,
    brace_type: &BraceType,
) -> Vec<Violation> {
    let text = context.get_span_text(span);
    if text.is_empty() || !text.starts_with('{') || !text.ends_with('}') {
        return vec![];
    }
    let inner = &text[1..text.len() - 1];
    if inner.trim().is_empty() {
        return vec![];
    }

    // Use global (AST) spans - they will be normalized later by the engine
    match brace_type {
        BraceType::ClosureWithParams => {
            if let Some(pipe_pos) = inner.find('|')
                && pipe_pos > 0
                && inner[..pipe_pos].chars().all(char::is_whitespace)
            {
                let opening_brace_span = Span::new(span.start, span.start + 1);
                vec![
                    Violation::new(
                        "No space allowed after opening brace before closure parameters"
                            .to_string(),
                        opening_brace_span,
                    )
                    .with_primary_label("opening brace")
                    .with_extra_span(Span::new(span.start + 1, span.start + 1 + pipe_pos))
                    .with_help("Use {|param| instead of { |param|"),
                ]
            } else {
                vec![]
            }
        }
        BraceType::BlockWithoutParams => {
            let starts_with_space = inner.starts_with(char::is_whitespace);
            let ends_with_space = inner.ends_with(char::is_whitespace);
            if !starts_with_space || !ends_with_space {
                let opening_span = Span::new(span.start, span.start + 1);
                let closing_span = Span::new(span.end - 1, span.end);
                vec![
                    Violation::new(
                        "Blocks and closures without parameters should have spaces inside braces"
                            .to_string(),
                        span,
                    )
                    .with_extra_label("needs space after", opening_span)
                    .with_extra_label("needs space before", closing_span)
                    .with_help("Use { body } for blocks without parameters"),
                ]
            } else {
                vec![]
            }
        }
        BraceType::Record => {
            if inner.contains('\n') {
                return vec![];
            }
            let starts_with_space = inner.starts_with(char::is_whitespace);
            let ends_with_space = inner.ends_with(char::is_whitespace);
            if starts_with_space || ends_with_space {
                let opening_span = Span::new(span.start, span.start + 1);
                let closing_span = Span::new(span.end - 1, span.end);
                vec![
                    Violation::new(
                        "Records should not have spaces inside braces".to_string(),
                        span,
                    )
                    .with_extra_label("no space after", opening_span)
                    .with_extra_label("no space before", closing_span)
                    .with_help("Use {key: value} for records"),
                ]
            } else {
                vec![]
            }
        }
    }
}
fn has_block_params(context: &LintContext, block_id: nu_protocol::BlockId) -> bool {
    let block = context.working_set.get_block(block_id);
    !block.signature.required_positional.is_empty()
        || !block.signature.optional_positional.is_empty()
        || block.signature.rest_positional.is_some()
}
fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Closure(block_id) | Expr::Block(block_id) => {
            let brace_type = if has_block_params(ctx, *block_id) {
                BraceType::ClosureWithParams
            } else {
                BraceType::BlockWithoutParams
            };
            check_brace_spacing(ctx, expr.span, &brace_type)
        }
        Expr::Record(items) if !items.is_empty() => {
            check_brace_spacing(ctx, expr.span, &BraceType::Record)
        }
        _ => vec![],
    })
}
pub const fn rule() -> Rule {
    Rule::new(
        "brace_spacing",
        "Enforces Nushell style guide: records use {key: value}, blocks/closures without params \
         use { body }, and closures with params use {|x| body}",
        check,
        LintLevel::Hint,
    )
    .with_doc_url("https://www.nushell.sh/book/style_guide.html#one-line-format")
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
