use nu_protocol::ast::{Expr, Expression};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, regex::contains_regex_special_chars, string::StringFormat},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct FixData {
    full_expr_span: nu_protocol::Span,
    string_span: nu_protocol::Span,
    pattern_span: nu_protocol::Span,
    is_negated: bool,
}

fn is_valid_pattern(pattern_expr: &Expression, context: &LintContext) -> bool {
    match &pattern_expr.expr {
        Expr::String(_) | Expr::RawString(_) => {
            let Some(string_format) = StringFormat::from_expression(pattern_expr, context) else {
                return false;
            };
            !contains_regex_special_chars(string_format.content())
        }
        Expr::Var(_) | Expr::VarDecl(_) | Expr::FullCellPath(_) => true,
        _ => false,
    }
}

fn extract_str_contains_data<'a>(
    expr: &'a Expression,
    context: &'a LintContext,
) -> Option<(&'a Expression, &'a Expression)> {
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

    Some((&first_element.expr, pattern_arg))
}

fn check_contains_pattern(expr: &Expression, context: &LintContext) -> Vec<(Detection, FixData)> {
    let (is_negated, inner_expr) = match &expr.expr {
        Expr::UnaryNot(inner) => (true, inner.as_ref()),
        _ => (false, expr),
    };

    let Some((string_expr, pattern_expr)) = extract_str_contains_data(inner_expr, context) else {
        return vec![];
    };

    if !is_valid_pattern(pattern_expr, context) {
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
    .with_primary_label(format!("verbose {verb} pattern"));

    let fix_data = FixData {
        full_expr_span: expr.span,
        string_span: string_expr.span,
        pattern_span: pattern_expr.span,
        is_negated,
    };

    vec![(violation, fix_data)]
}

struct UseRegexOperators;

impl DetectFix for UseRegexOperators {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "contains_to_regex_op"
    }

    fn short_description(&self) -> &'static str {
        "Use =~ and !~ operators instead of verbose 'str contains' checks"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/operators.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(check_contains_pattern)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let string_text = context.span_text(fix_data.string_span);
        let pattern_text = context.span_text(fix_data.pattern_span);

        let operator = if fix_data.is_negated { "!~" } else { "=~" };
        let fixed_text = format!("{string_text} {operator} {pattern_text}");

        let explanation = if fix_data.is_negated {
            "Replace negated str contains check with !~ operator"
        } else {
            "Replace str contains check with =~ operator"
        };

        Some(Fix {
            explanation: explanation.into(),
            replacements: vec![Replacement::new(fix_data.full_expr_span, fixed_text)],
        })
    }
}

pub static RULE: &dyn Rule = &UseRegexOperators;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
