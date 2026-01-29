use nu_protocol::{
    Span,
    ast::{Expr, Expression, ExternalArgument},
};

use crate::{
    LintLevel,
    context::LintContext,
    dsl::{ConversionContext, jq},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct JqFixData {
    expr_span: Span,
    conversion: jq::NuEquivalent,
    context: ConversionContext,
}

fn try_convert_jq_call<'a>(
    expr: &'a Expression,
    ctx: &'a LintContext,
) -> Option<(Detection, JqFixData)> {
    let Expr::ExternalCall(head, args) = &expr.expr else {
        return None;
    };

    if ctx.span_text(head.span) != "jq" {
        return None;
    }

    let arg_exprs: Vec<&Expression> = args
        .iter()
        .map(|arg| match arg {
            ExternalArgument::Regular(e) | ExternalArgument::Spread(e) => e,
        })
        .collect();

    let arg_texts: Vec<&str> = arg_exprs
        .iter()
        .map(|e| match &e.expr {
            Expr::String(s) | Expr::RawString(s) => s.as_str(),
            _ => ctx.expr_text(e),
        })
        .collect();

    let filter_index = arg_texts
        .iter()
        .position(|arg| !arg.starts_with('-'))
        .unwrap_or(0);

    let filter_expr = arg_exprs.get(filter_index)?;

    let conv_ctx = arg_exprs
        .get(filter_index + 1)
        .map_or(ConversionContext::Pipeline, |file| {
            ConversionContext::File(ctx.expr_text(file).to_string())
        });

    // AST-based detection: if convert returns None, the pattern is unsupported
    let conversion = if let Expr::StringInterpolation(exprs) = &filter_expr.expr {
        jq::convert_interpolation(exprs, ctx)?
    } else {
        let filter = arg_texts.get(filter_index)?;
        if filter.is_empty() {
            return None;
        }
        jq::convert(filter)?
    };

    let detection = Detection::from_global_span(
        "Use built-in Nushell commands for simple operations - they're faster and more idiomatic",
        expr.span,
    )
    .with_primary_label("external `jq`");

    Some((
        detection,
        JqFixData {
            expr_span: expr.span,
            conversion,
            context: conv_ctx,
        },
    ))
}

struct ReplaceJqWithNuGet;

impl DetectFix for ReplaceJqWithNuGet {
    type FixInput<'a> = JqFixData;

    fn id(&self) -> &'static str {
        "jq_to_nu_pipeline"
    }

    fn short_description(&self) -> &'static str {
        "Simple `jq` filter replaceable with Nushell pipeline"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Detects external `jq` calls with filters that can be expressed as native Nushell \
             pipelines. Uses the jaq parser to analyze jq filter syntax and only reports when an \
             equivalent Nushell command exists.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/cookbook/jq_v_nushell.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .detect_with_fix_data(|expr, ctx| try_convert_jq_call(expr, ctx).into_iter().collect())
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let nu_cmd = fix_data.conversion.format(&fix_data.context, context);
        Some(Fix {
            explanation: "Replace jq filter with equivalent Nushell pipeline".into(),
            replacements: vec![Replacement::new(fix_data.expr_span, nu_cmd)],
        })
    }
}

pub static RULE: &dyn Rule = &ReplaceJqWithNuGet;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
