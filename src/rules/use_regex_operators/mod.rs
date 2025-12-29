use nu_protocol::ast::{Expr, Expression};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, regex::contains_regex_special_chars, string::strip_quotes},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct FixData {
    full_expr_span: nu_protocol::Span,
    string_expr_span: nu_protocol::Span,
    pattern_span: nu_protocol::Span,
    is_negated: bool,
}

fn is_simple_literal_pattern(pattern_text: &str) -> bool {
    let unquoted = strip_quotes(pattern_text);
    !contains_regex_special_chars(unquoted)
}

fn extract_str_contains_spans(
    expr: &Expression,
    context: &LintContext,
) -> Option<(nu_protocol::Span, nu_protocol::Span)> {
    let block_id = match &expr.expr {
        Expr::Subexpression(id) => *id,
        Expr::FullCellPath(path) => {
            if let Expr::Subexpression(id) = &path.head.expr {
                *id
            } else {
                return None;
            }
        }
        _ => return None,
    };

    let block = context.working_set.get_block(block_id);
    let pipeline = block.pipelines.first()?;

    if pipeline.elements.len() < 2 {
        return None;
    }

    let last_element = pipeline.elements.last()?;
    let Expr::Call(call) = &last_element.expr.expr else {
        return None;
    };

    if call.get_call_name(context) != "str contains" {
        return None;
    }

    let pattern_arg = call.get_first_positional_arg()?;
    let first_element = pipeline.elements.first()?;

    Some((first_element.expr.span, pattern_arg.span))
}

fn check_contains_pattern(expr: &Expression, context: &LintContext) -> Vec<(Detection, FixData)> {
    let (is_negated, inner_expr) = match &expr.expr {
        Expr::UnaryNot(inner) => (true, inner.as_ref()),
        _ => (false, expr),
    };

    let Some((string_expr_span, pattern_span)) = extract_str_contains_spans(inner_expr, context)
    else {
        return vec![];
    };

    let pattern_text = context.get_span_text(pattern_span);
    if !is_simple_literal_pattern(pattern_text) {
        return vec![];
    }

    let (operator, verb) = if is_negated {
        ("!~", "negated str contains")
    } else {
        ("=~", "str contains")
    };

    let violation = Detection::from_global_span(
        format!("Use '{operator}' operator instead of verbose '{verb}' check"),
        expr.span,
    )
    .with_primary_label(format!("verbose {verb} pattern"))
    .with_help(format!(
        "The '{operator}' operator is more concise and idiomatic for simple pattern matching"
    ));

    let fix_data = FixData {
        full_expr_span: expr.span,
        string_expr_span,
        pattern_span,
        is_negated,
    };

    vec![(violation, fix_data)]
}

struct UseRegexOperators;

impl DetectFix for UseRegexOperators {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "use_regex_operators"
    }

    fn explanation(&self) -> &'static str {
        "Use =~ and !~ operators instead of verbose 'str contains' checks"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/operators.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(check_contains_pattern)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let string_text = context.get_span_text(fix_data.string_expr_span);
        let pattern_text = context.get_span_text(fix_data.pattern_span);

        let operator = if fix_data.is_negated { "!~" } else { "=~" };
        let fixed_text = format!("{string_text} {operator} {pattern_text}");

        let explanation = if fix_data.is_negated {
            "Replace negated str contains check with !~ operator"
        } else {
            "Replace str contains check with =~ operator"
        };

        Some(Fix::with_explanation(
            explanation,
            vec![Replacement::new(fix_data.full_expr_span, fixed_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &UseRegexOperators;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
