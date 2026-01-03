use nu_protocol::ast::{Expr, Expression};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    interpolation_span: nu_protocol::Span,
    variable_text: String,
}

fn is_single_variable_interpolation(exprs: &[Expression], context: &LintContext) -> Option<String> {
    if exprs.len() != 1 {
        return None;
    }

    let expr = &exprs[0];

    let inner_expr = match &expr.expr {
        Expr::FullCellPath(cell_path) => &cell_path.head,
        _ => expr,
    };

    match &inner_expr.expr {
        Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);

            if block.pipelines.len() != 1 || block.pipelines[0].elements.len() != 1 {
                return None;
            }

            let pipeline_expr = &block.pipelines[0].elements[0].expr;

            match &pipeline_expr.expr {
                Expr::Var(_) | Expr::VarDecl(_) | Expr::FullCellPath(_) => {
                    let var_text = context.plain_text(pipeline_expr.span);
                    Some(var_text.to_string())
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn check_interpolation(expr: &Expression, context: &LintContext) -> Option<(Detection, FixData)> {
    let Expr::StringInterpolation(exprs) = &expr.expr else {
        return None;
    };

    let variable_text = is_single_variable_interpolation(exprs, context)?;

    let violation = Detection::from_global_span(
        format!("Unnecessary string interpolation around variable '{variable_text}'"),
        expr.span,
    )
    .with_primary_label("unnecessary interpolation")
    .with_help(format!(
        "Nushell never splits variables on whitespace, unlike bash.\nYou can use the variable \
         directly: {variable_text}\nQuotes are only needed to prevent splitting in bash, not in \
         Nushell."
    ));

    let fix_data = FixData {
        interpolation_span: expr.span,
        variable_text,
    };

    Some((violation, fix_data))
}

struct NeverSpaceSplit;

impl DetectFix for NeverSpaceSplit {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "never_space_split"
    }

    fn explanation(&self) -> &'static str {
        "Nushell never splits variables on whitespace; quotes around single variables are \
         unnecessary"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/variables.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .detect_with_fix_data(|expr, ctx| check_interpolation(expr, ctx).into_iter().collect())
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            format!(
                "Remove unnecessary interpolation around '{}'",
                fix_data.variable_text
            ),
            vec![Replacement::new(
                fix_data.interpolation_span,
                fix_data.variable_text.clone(),
            )],
        ))
    }
}

pub static RULE: &dyn Rule = &NeverSpaceSplit;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
