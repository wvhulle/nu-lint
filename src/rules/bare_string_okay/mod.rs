use nu_protocol::ast::{Expr, Expression};

use crate::{
    LintLevel,
    ast::string::{StringFormat, bare_word_needs_quotes},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    quoted_span: nu_protocol::Span,
    unquoted_content: String,
}

fn check_string_needs_quotes(expr: &Expression, ctx: &LintContext) -> Option<(Detection, FixData)> {
    let string_format = StringFormat::from_expression(expr, ctx)?;

    let content = match &string_format {
        StringFormat::Double(s) | StringFormat::Single(s) => s,
        StringFormat::BareWord(_)
        | StringFormat::InterpolationDouble(_)
        | StringFormat::InterpolationSingle(_)
        | StringFormat::Backtick(_)
        | StringFormat::Raw(_) => return None,
    };

    if bare_word_needs_quotes(content) {
        return None;
    }

    let quote_type = match string_format {
        StringFormat::Double(_) => "double",
        StringFormat::Single(_) => "single",
        _ => return None,
    };

    let violation = Detection::from_global_span(
        format!("Unnecessary {quote_type} quotes around string '{content}'"),
        expr.span,
    )
    .with_primary_label("unnecessary quotes")
    .with_help(
        "In Nushell, simple strings without spaces or special characters can be written as bare \
         words."
            .to_string(),
    );

    let fix_data = FixData {
        quoted_span: expr.span,
        unquoted_content: content.clone(),
    };

    Some((violation, fix_data))
}

struct BareStringOkay;

impl DetectFix for BareStringOkay {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "bare_string_okay"
    }

    fn explanation(&self) -> &'static str {
        "Simple strings without spaces or special characters can be written as bare words"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/working_with_strings.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            if !matches!(expr.expr, Expr::String(_)) {
                return vec![];
            }

            check_string_needs_quotes(expr, ctx).into_iter().collect()
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            format!("Remove quotes from '{}'", fix_data.unquoted_content),
            vec![Replacement::new(
                fix_data.quoted_span,
                fix_data.unquoted_content.clone(),
            )],
        ))
    }
}

pub static RULE: &dyn Rule = &BareStringOkay;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
