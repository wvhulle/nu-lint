use nu_protocol::{
    BlockId, Span, SyntaxShape,
    ast::{Call, Expr},
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
}

fn inner_list_type(shape: &SyntaxShape) -> Option<SyntaxShape> {
    match shape {
        SyntaxShape::List(inner) if !matches!(inner.as_ref(), SyntaxShape::List(_)) => {
            Some(*inner.clone())
        }
        _ => None,
    }
}

fn detect_in_def(call: &Call, ctx: &LintContext) -> Vec<(Detection, FixData)> {
    let Some(_) = call.custom_command_def(ctx) else {
        return vec![];
    };
    let Some(sig_expr) = call.get_positional_arg(1) else {
        return vec![];
    };
    let Some(body_expr) = call.get_positional_arg(2) else {
        return vec![];
    };
    let Expr::Signature(sig) = &sig_expr.expr else {
        return vec![];
    };
    let Some(body_block_id) = body_expr.extract_block_id() else {
        return vec![];
    };

    // Skip if already has variadic
    if sig.rest_positional.is_some() {
        return vec![];
    }

    // Get last positional (optional takes precedence over required)
    let last_positional = sig
        .optional_positional
        .last()
        .or_else(|| sig.required_positional.last());

    let Some(param) = last_positional else {
        return vec![];
    };

    // Must be a simple list type (not nested)
    let Some(inner_type) = inner_list_type(&param.shape) else {
        return vec![];
    };

    let signature_span = sig_expr.span;
    let param_span = signature_span.find_substring_span(&param.name, ctx);
    let type_span = signature_span.find_substring_span(&param.shape.to_string(), ctx);

    let detection = Detection::from_global_span(
        format!(
            "Parameter `{}` could be variadic `...{}` for better CLI ergonomics",
            param.name, param.name
        ),
        param_span,
    )
    .with_primary_label("last positional parameter")
    .with_extra_label(format!("has list type `{}`", param.shape), type_span)
    .with_extra_label(
        format!("use `...{}: {inner_type}` instead", param.name),
        signature_span,
    );

    vec![(
        detection,
        FixData {
            signature_span,
            body_block_id,
        },
    )]
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
            Expr::Call(call) => detect_in_def(call, ctx),
            _ => vec![],
        })
    }

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let fix_data: &FixData = fix_data;
        let block = ctx.working_set.get_block(fix_data.body_block_id);
        let sig = &block.signature;

        let last_required_is_list = sig.optional_positional.is_empty()
            && sig
                .required_positional
                .last()
                .is_some_and(|p| inner_list_type(&p.shape).is_some());

        let last_optional_is_list = sig
            .optional_positional
            .last()
            .is_some_and(|p| inner_list_type(&p.shape).is_some());

        let mut parts = Vec::new();

        // Format required params (convert last one if it's the list param)
        for (i, p) in sig.required_positional.iter().enumerate() {
            let is_last = i == sig.required_positional.len() - 1;
            if is_last && last_required_is_list {
                let inner = inner_list_type(&p.shape).unwrap_or(SyntaxShape::Any);
                parts.push(format_rest_with_shape(&p.name, &inner));
            } else {
                parts.push(format_required(p));
            }
        }

        // Format optional params (convert last one if it's the list param)
        for (i, p) in sig.optional_positional.iter().enumerate() {
            let is_last = i == sig.optional_positional.len() - 1;
            if is_last && last_optional_is_list {
                let inner = inner_list_type(&p.shape).unwrap_or(SyntaxShape::Any);
                parts.push(format_rest_with_shape(&p.name, &inner));
            } else {
                parts.push(format_optional(p));
            }
        }

        // Format flags (skip auto-generated --help)
        for f in &sig.named {
            if f.long != "help" {
                parts.push(format_flag(f));
            }
        }

        let new_signature = format!("[{}]", parts.join(", "));

        Some(Fix::with_explanation(
            "Convert list parameter to variadic",
            vec![Replacement::new(fix_data.signature_span, new_signature)],
        ))
    }
}

pub static RULE: &dyn Rule = &ListParamToVariadic;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
