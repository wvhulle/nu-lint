use std::cmp::Ordering;

use nu_protocol::{
    Span,
    ast::{Argument, Block, Expr, Expression, Pipeline, PipelineElement, Traverse},
};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{block::BlockExt, declaration::CustomCommandDef, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

/// Check if a block has a single pipeline suitable for inlining.
fn is_inlinable_body(block: &Block, context: &LintContext) -> bool {
    // Must have exactly one non-empty pipeline
    let non_empty_pipeline_count = block
        .pipelines
        .iter()
        .filter(|p| {
            p.elements
                .iter()
                .any(|e| !matches!(&e.expr.expr, Expr::Nothing))
        })
        .count();

    if non_empty_pipeline_count != 1 {
        return false;
    }

    // Body must fit within 3 lines
    block
        .span
        .is_none_or(|span| context.span_text(span).lines().count() <= 3)
}

/// Extract the actual pipeline elements from a body.
/// Handles the `Collect(Subexpression(...))` pattern used for `$in | ...`
/// bodies.
fn unwrap_body_pipeline<'a>(
    pipeline: &'a Pipeline,
    context: &'a LintContext,
) -> Option<(&'a [PipelineElement], &'a Expression, bool)> {
    let first = pipeline.elements.first()?;

    // Detect Collect(Subexpression) pattern: `{ $in | something }`
    if let Expr::Collect(_, inner) = &first.expr.expr
        && let Expr::Subexpression(block_id) = &inner.expr
    {
        let inner_pipeline = context.working_set.get_block(*block_id).pipelines.first()?;
        return Some((&inner_pipeline.elements, &first.expr, true));
    }

    Some((&pipeline.elements, &first.expr, false))
}

/// Find the single call site for a function by declaration ID.
fn find_single_call(
    decl_id: nu_protocol::DeclId,
    context: &LintContext,
) -> Option<(Span, Vec<Span>)> {
    let mut calls = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };
            if call.decl_id != decl_id {
                return vec![];
            }

            let args = call
                .arguments
                .iter()
                .filter_map(|a| match a {
                    Argument::Positional(e) | Argument::Unknown(e) => Some(e.span),
                    _ => None,
                })
                .collect();
            vec![(expr.span, args)]
        },
        &mut calls,
    );

    (calls.len() == 1).then(|| calls.into_iter().next())?
}

/// A text substitution to apply when inlining. Ordered by span start.
#[derive(Eq, PartialEq)]
struct Substitution<'a> {
    span: Span,
    replacement: &'a str,
}

impl PartialOrd for Substitution<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Substitution<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.span.start.cmp(&other.span.start)
    }
}

/// Data needed to generate the inline fix
struct FixData {
    definition_span: Span,
    call_span: Span,
    body_span: Span,
    body_start: usize,
    substitutions: Vec<(Span, Span)>, // (usage_span, arg_span)
}

struct InlineSingleUseFunction;

impl DetectFix for InlineSingleUseFunction {
    type FixInput<'a> = Option<FixData>;

    fn id(&self) -> &'static str {
        "single_call_command"
    }

    fn short_description(&self) -> &'static str {
        "Single-line command called only once"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let commands = context.custom_commands();
        if !commands.iter().any(CustomCommandDef::is_main) {
            return vec![];
        }

        commands
            .iter()
            .filter(|def| !def.is_main() && !def.is_exported())
            .filter_map(|def| {
                let block = context.working_set.get_block(def.body);
                if !is_inlinable_body(block, context) {
                    return None;
                }

                let decl_id = context.working_set.find_decl(def.name.as_bytes())?;
                let (call_span, call_args) = find_single_call(decl_id, context)?;

                let pipeline = block.pipelines.iter().find(|p| {
                    p.elements
                        .iter()
                        .any(|e| !matches!(&e.expr.expr, Expr::Nothing))
                })?;

                let (elements, _, has_dollar_in) = unwrap_body_pipeline(pipeline, context)?;
                let first = elements.first()?;
                let last = elements.last()?;
                let body_span = Span::new(first.expr.span.start, last.expr.span.end);

                // Skip `$in | ` prefix if present
                let body_start = if has_dollar_in {
                    elements.get(1).map(|e| e.expr.span.start)
                } else {
                    first
                        .expr
                        .find_dollar_in_usage()
                        .filter(|s| s.start == first.expr.span.start)
                        .and_then(|_| elements.get(1).map(|e| e.expr.span.start))
                }
                .unwrap_or(body_span.start);

                // Map parameters to their argument spans (all usages)
                let params = def
                    .signature
                    .required_positional
                    .iter()
                    .chain(&def.signature.optional_positional);
                let substitutions: Vec<_> = params
                    .zip(&call_args)
                    .filter_map(|(param, &arg_span)| {
                        let idx = def
                            .signature
                            .required_positional
                            .iter()
                            .chain(&def.signature.optional_positional)
                            .position(|p| p.name == param.name)?;
                        let var_id = block.signature.get_positional(idx)?.var_id?;
                        let usage_spans = block.var_usages(var_id, context, |_, _, _| true);
                        Some(
                            usage_spans
                                .into_iter()
                                .map(move |usage_span| (usage_span, arg_span)),
                        )
                    })
                    .flatten()
                    .collect();

                let detection = Detection::from_file_span(
                    format!(
                        "Function `{}` has a single-line body and is only used once",
                        def.name
                    ),
                    def.declaration_span(context),
                )
                .with_primary_label("single-use function")
                .with_extra_label("could be inlined", block.span?);

                Some((
                    detection,
                    Some(FixData {
                        definition_span: context.expand_span_to_full_lines(def.definition_span),
                        call_span,
                        body_span,
                        body_start,
                        substitutions,
                    }),
                ))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let data = fix_data.as_ref()?;
        let body_span = Span::new(data.body_start, data.body_span.end);
        let body_text = context.span_text(body_span);

        // Collect and sort substitutions, apply in reverse order to preserve span
        // validity
        let mut subs: Vec<_> = data
            .substitutions
            .iter()
            .filter(|(usage, _)| usage.start >= body_span.start && usage.end <= body_span.end)
            .map(|(usage, arg)| Substitution {
                span: Span::new(usage.start - body_span.start, usage.end - body_span.start),
                replacement: context.span_text(*arg),
            })
            .collect();
        subs.sort();

        let inlined = subs
            .iter()
            .rev()
            .fold(body_text.to_string(), |mut text, sub| {
                text.replace_range(sub.span.start..sub.span.end, sub.replacement);
                text
            });

        Some(Fix {
            explanation: "Inline function body and remove definition".into(),
            replacements: vec![
                Replacement::new(data.call_span, inlined),
                Replacement::new(data.definition_span, String::new()),
            ],
        })
    }
}

pub static RULE: &dyn Rule = &InlineSingleUseFunction;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
