use std::collections::HashSet;

use nu_protocol::{Span, ast::Expr};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    span::LintSpan,
    violation::{Detection, Fix, Replacement},
};

#[derive(Clone)]
enum BraceType {
    ClosureWithParams,
    BlockWithoutParams,
    Record,
}

struct BraceSpacingFixData {
    span: Span,
    brace_type: BraceType,
}

fn check_brace_spacing(
    context: &LintContext,
    span: Span,
    brace_type: &BraceType,
) -> Vec<(Detection, BraceSpacingFixData)> {
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
                vec![(
                    Detection::from_global_span(
                        "No space allowed after opening brace before closure parameters"
                            .to_string(),
                        opening_brace_span,
                    )
                    .with_primary_label("opening brace")
                    .with_extra_span(Span::new(span.start + 1, span.start + 1 + pipe_pos))
                    .with_help("Use {|param| instead of { |param|"),
                    BraceSpacingFixData {
                        span,
                        brace_type: brace_type.clone(),
                    },
                )]
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
                vec![(
                    Detection::from_global_span(
                        "Blocks and closures without parameters should have spaces inside braces"
                            .to_string(),
                        span,
                    )
                    .with_extra_label("needs space after", opening_span)
                    .with_extra_label("needs space before", closing_span)
                    .with_help("Use { body } for blocks without parameters"),
                    BraceSpacingFixData {
                        span,
                        brace_type: brace_type.clone(),
                    },
                )]
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
                vec![(
                    Detection::from_global_span(
                        "Records should not have spaces inside braces".to_string(),
                        span,
                    )
                    .with_extra_label("no space after", opening_span)
                    .with_extra_label("no space before", closing_span)
                    .with_help("Use {key: value} for records"),
                    BraceSpacingFixData {
                        span,
                        brace_type: brace_type.clone(),
                    },
                )]
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

const fn is_record_type(ty: &nu_protocol::Type) -> bool {
    matches!(ty, nu_protocol::Type::Record(_))
}

struct BraceSpacing;

impl DetectFix for BraceSpacing {
    type FixInput<'a> = BraceSpacingFixData;

    fn id(&self) -> &'static str {
        "brace_spacing"
    }

    fn explanation(&self) -> &'static str {
        "Enforce consistent brace spacing per Nushell style guide"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#one-line-format")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut seen_spans: HashSet<(usize, usize)> = HashSet::new();
        let results = context.detect_with_fix_data(|expr, ctx| {
            match &expr.expr {
                Expr::Closure(block_id) | Expr::Block(block_id) => {
                    // If the expression type is Record, treat it as a record (not a block)
                    // Nushell parses record literals in variable assignments as Block with Record
                    // type
                    if is_record_type(&expr.ty) {
                        return check_brace_spacing(ctx, expr.span, &BraceType::Record);
                    }

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
            }
        });

        // Deduplicate by span - the same record can be visited via multiple AST paths
        results
            .into_iter()
            .filter(|(detection, _)| {
                let span_key = match detection.span {
                    LintSpan::Global(s) => (s.start, s.end),
                    LintSpan::File(s) => (s.start, s.end),
                };
                seen_spans.insert(span_key)
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let text = context.get_span_text(fix_data.span);
        let inner = &text[1..text.len() - 1];

        let fixed = match &fix_data.brace_type {
            BraceType::ClosureWithParams => {
                // Remove space after opening brace
                let trimmed = inner.trim_start();
                format!("{{{trimmed}}}")
            }
            BraceType::BlockWithoutParams => {
                // Add spaces if missing
                let trimmed = inner.trim();
                format!("{{ {trimmed} }}")
            }
            BraceType::Record => {
                // Remove spaces
                let trimmed = inner.trim();
                format!("{{{trimmed}}}")
            }
        };

        Some(Fix::with_explanation(
            "Fix brace spacing",
            vec![Replacement::new(fix_data.span, fixed)],
        ))
    }
}

pub static RULE: &dyn Rule = &BraceSpacing;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
