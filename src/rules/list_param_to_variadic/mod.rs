use std::iter::once;

use nu_protocol::{
    BlockId, DeclId, Span, SyntaxShape,
    ast::{Argument, Call, Expr, Expression, ListItem, Traverse},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    rules::typing::{format_flag, format_optional, format_required, format_rest_with_shape},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    signature_span: Span,
    body_block_id: BlockId,
    decl_id: DeclId,
    param_index: usize,
}

struct ListItemSpan {
    span: Span,
    is_spread: bool,
}

/// Find all call sites and return replacements for their list arguments
fn find_call_site_replacements(
    decl_id: DeclId,
    param_index: usize,
    ctx: &LintContext,
) -> Vec<Replacement> {
    let mut replacements = Vec::new();

    ctx.ast.flat_map(
        ctx.working_set,
        &|expr| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };
            if call.decl_id != decl_id {
                return vec![];
            }

            // Get nth positional argument (Some(expr) for regular, None for spread)
            call.arguments
                .iter()
                .filter_map(|arg| match arg {
                    Argument::Positional(e) | Argument::Unknown(e) => Some(Some(e)),
                    Argument::Spread(_) => Some(None), // counts as positional slot but skip
                    // transform
                    Argument::Named(_) => None,
                })
                .nth(param_index)
                .flatten()
                .map(|e| vec![transform_arg(e, ctx)])
                .unwrap_or_default()
        },
        &mut replacements,
    );

    replacements
}

/// Transform an argument expression to variadic format
fn transform_arg(expr: &Expression, ctx: &LintContext) -> Replacement {
    let replacement_text = extract_list_items(expr).map_or_else(
        || format!("...{}", ctx.span_text(expr.span)),
        |items| {
            items
                .iter()
                .map(|item| {
                    let text = ctx.span_text(item.span);
                    if item.is_spread {
                        format!("...{text}")
                    } else {
                        text.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        },
    );
    Replacement::new(expr.span, replacement_text)
}

/// Extract list items if expression is a list literal
fn extract_list_items(expr: &Expression) -> Option<Vec<ListItemSpan>> {
    let items = match &expr.expr {
        Expr::List(items) => items,
        Expr::FullCellPath(fcp) if fcp.tail.is_empty() => {
            if let Expr::List(items) = &fcp.head.expr {
                items
            } else {
                return None;
            }
        }
        _ => return None,
    };
    Some(
        items
            .iter()
            .map(|item| match item {
                ListItem::Item(e) => ListItemSpan {
                    span: e.span,
                    is_spread: false,
                },
                ListItem::Spread(_, e) => ListItemSpan {
                    span: e.span,
                    is_spread: true,
                },
            })
            .collect(),
    )
}

fn inner_list_type(shape: &SyntaxShape) -> Option<SyntaxShape> {
    match shape {
        SyntaxShape::List(inner) if !matches!(inner.as_ref(), SyntaxShape::List(_)) => {
            Some(*inner.clone())
        }
        _ => None,
    }
}

fn detect_in_def(call: &Call, ctx: &LintContext) -> Option<(Detection, FixData)> {
    let cmd_def = call.custom_command_def(ctx)?;
    let sig_expr = call.get_positional_arg(1)?;
    let Expr::Signature(sig) = &sig_expr.expr else {
        return None;
    };
    let body_block_id = call.get_positional_arg(2)?.extract_block_id()?;
    sig.rest_positional.is_none().then_some(())?;

    // Find last positional with list type (optional takes precedence over required)
    let (param, param_index) = sig
        .optional_positional
        .last()
        .filter(|p| inner_list_type(&p.shape).is_some())
        .map(|p| {
            (
                p,
                sig.required_positional.len() + sig.optional_positional.len() - 1,
            )
        })
        .or_else(|| {
            sig.optional_positional.is_empty().then_some(())?;
            let p = sig
                .required_positional
                .last()
                .filter(|p| inner_list_type(&p.shape).is_some())?;
            Some((p, sig.required_positional.len() - 1))
        })?;

    let inner_type = inner_list_type(&param.shape)?;
    let decl_id = ctx.working_set.find_decl(cmd_def.name.as_bytes())?;
    let signature_span = sig_expr.span;

    let detection = Detection::from_global_span(
        format!(
            "Parameter `{}` could be variadic `...{}` for better CLI ergonomics",
            param.name, param.name
        ),
        signature_span.find_substring_span(&param.name, ctx),
    )
    .with_primary_label("last positional parameter")
    .with_extra_label(
        format!("has list type `{}`", param.shape),
        signature_span.find_substring_span(&param.shape.to_string(), ctx),
    )
    .with_extra_label(
        format!("use `...{}: {inner_type}` instead", param.name),
        signature_span,
    );

    Some((
        detection,
        FixData {
            signature_span,
            body_block_id,
            decl_id,
            param_index,
        },
    ))
}

struct ListParamToVariadic;

impl DetectFix for ListParamToVariadic {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "list_param_to_variadic"
    }

    fn short_description(&self) -> &'static str {
        "Use variadic `...args` instead of a single list parameter"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "This will make the command easier to use in situations where you call the function \
             directly from the command-line as a script. In such cases, the nu function that is \
             the entry point of the script will treat a list received from command-line as a \
             single string, even when you typed it as a list. Using variadic arguments prevents \
             this problem.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#rest-parameters")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::Call(call) => detect_in_def(call, ctx).into_iter().collect(),
            _ => vec![],
        })
    }

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let sig = &ctx.working_set.get_block(fix_data.body_block_id).signature;

        let format_positional = |i: usize, p: &nu_protocol::PositionalArg, is_optional: bool| {
            if i == fix_data.param_index {
                let inner = inner_list_type(&p.shape).unwrap_or(SyntaxShape::Any);
                format_rest_with_shape(&p.name, &inner)
            } else if is_optional {
                format_optional(p)
            } else {
                format_required(p)
            }
        };

        let parts: Vec<_> =
            sig.required_positional
                .iter()
                .enumerate()
                .map(|(i, p)| format_positional(i, p, false))
                .chain(
                    sig.optional_positional.iter().enumerate().map(|(i, p)| {
                        format_positional(sig.required_positional.len() + i, p, true)
                    }),
                )
                .chain(
                    sig.named
                        .iter()
                        .filter(|f| f.long != "help")
                        .map(format_flag),
                )
                .collect();

        let call_site_replacements =
            find_call_site_replacements(fix_data.decl_id, fix_data.param_index, ctx);

        let replacements: Vec<_> = once(Replacement::new(
            fix_data.signature_span,
            format!("[{}]", parts.join(", ")),
        ))
        .chain(call_site_replacements.iter().cloned())
        .collect();

        let explanation = if call_site_replacements.is_empty() {
            "Convert list parameter to variadic"
        } else {
            "Convert list parameter to variadic and update call sites"
        };

        Some(Fix::with_explanation(explanation, replacements))
    }
}

pub static RULE: &dyn Rule = &ListParamToVariadic;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
