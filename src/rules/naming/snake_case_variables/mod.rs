use heck::ToSnakeCase;
use nu_protocol::{
    Span, VarId,
    ast::{Argument, Call, Expr},
};

use crate::{
    LintLevel,
    ast::span::SpanExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct SnakeCaseFixData {
    var_name: String,
    snake_case_name: String,
    replacements: Vec<(Span, String)>,
}

/// Find all usages of a variable in the AST
fn find_variable_usages(var_id: VarId, context: &LintContext) -> Vec<Span> {
    use nu_protocol::ast::Traverse;

    let mut usages = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| match &expr.expr {
            Expr::Var(id) if *id == var_id => vec![expr.span],
            Expr::FullCellPath(cell_path) if matches!(&cell_path.head.expr, Expr::Var(id) if *id == var_id) => {
                vec![cell_path.head.span]
            }
            _ => vec![],
        },
        &mut usages,
    );
    usages
}

fn check_call(call: &Call, ctx: &LintContext) -> Option<(Detection, SnakeCaseFixData)> {
    let decl_name = ctx.working_set.get_decl(call.decl_id).name();
    let is_mutable = matches!(decl_name, "mut");
    if !matches!(decl_name, "let" | "mut") {
        return None;
    }

    let Argument::Positional(name_expr) = call.arguments.first()? else {
        return None;
    };

    let Expr::VarDecl(var_id) = &name_expr.expr else {
        return None;
    };

    let var_name = ctx.get_span_text(name_expr.span);
    let snake_case_name = var_name.to_snake_case();

    if var_name == snake_case_name {
        return None;
    }

    // Collect replacement spans and texts
    let mut replacements = vec![(name_expr.span, snake_case_name.clone())];

    for usage_span in find_variable_usages(*var_id, ctx) {
        if usage_span.source_code(ctx).starts_with('$') {
            replacements.push((usage_span, format!("${snake_case_name}")));
        }
    }

    let var_type = if is_mutable {
        "Mutable variable"
    } else {
        "Variable"
    };

    let violation = Detection::from_global_span(
        format!("{var_type} '{var_name}' should use snake_case naming convention"),
        name_expr.span,
    )
    .with_primary_label("non-snake_case name")
    .with_help(format!("Consider renaming to: {snake_case_name}"));

    let fix_data = SnakeCaseFixData {
        var_name: var_name.to_string(),
        snake_case_name,
        replacements,
    };

    Some((violation, fix_data))
}

struct SnakeCaseVariables;

impl DetectFix for SnakeCaseVariables {
    type FixInput<'a> = SnakeCaseFixData;

    fn id(&self) -> &'static str {
        "snake_case_variables"
    }

    fn explanation(&self) -> &'static str {
        "Variables should use snake_case naming convention"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#variables-and-command-parameters")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            check_call(call, ctx).into_iter().collect()
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacements = fix_data
            .replacements
            .iter()
            .map(|(span, text)| Replacement::new(*span, text.clone()))
            .collect();
        Some(Fix::with_explanation(
            format!(
                "Rename variable '{}' to '{}'",
                fix_data.var_name, fix_data.snake_case_name
            ),
            replacements,
        ))
    }
}

pub static RULE: &dyn Rule = &SnakeCaseVariables;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
