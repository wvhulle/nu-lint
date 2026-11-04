use nu_lint_analysis::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::ast::{Expr, Expression};

struct FixData {
    interpolation_span: nu_protocol::Span,
    variable_span: nu_protocol::Span,
}

fn single_variable_span(exprs: &[Expression], context: &LintContext) -> Option<nu_protocol::Span> {
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
                Expr::Var(_) | Expr::VarDecl(_) | Expr::FullCellPath(_) => Some(pipeline_expr.span),
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

    let variable_span = single_variable_span(exprs, context)?;
    let variable_text = context.span_text(variable_span);

    let violation = Detection::from_global_span(
        format!("Unnecessary string interpolation around variable '{variable_text}'"),
        expr.span,
    )
    .with_primary_label("unnecessary interpolation");

    let fix_data = FixData {
        interpolation_span: expr.span,
        variable_span,
    };

    Some((violation, fix_data))
}

struct NeverSpaceSplit;

impl DetectFix for NeverSpaceSplit {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "never_space_split"
    }

    fn short_description(&self) -> &'static str {
        "Unnecessary quotes around variable"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/variables.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .detect_with_fix_data(|expr, ctx| check_interpolation(expr, ctx).into_iter().collect())
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let variable_text = context.span_text(fix_data.variable_span);
        Some(Fix {
            explanation: format!("Remove unnecessary interpolation around '{variable_text}'")
                .into(),
            replacements: vec![Replacement::new(
                fix_data.interpolation_span,
                variable_text.to_string(),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &NeverSpaceSplit;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
