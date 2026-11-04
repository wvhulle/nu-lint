use heck::ToSnakeCase;
use nu_lint_analysis::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Span, VarId,
    ast::{Argument, Call, Expr},
};

struct SnakeCaseFixData {
    var_name_span: Span,
    usage_spans: Vec<Span>,
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
    let builtin_name = call.builtin_name(ctx);
    let is_mutable = builtin_name == Some("mut");
    if !matches!(builtin_name, Some("let" | "mut")) {
        return None;
    }

    let Argument::Positional(name_expr) = call.arguments.first()? else {
        return None;
    };

    let Expr::VarDecl(var_id) = &name_expr.expr else {
        return None;
    };

    let var_name = ctx.expr_text(name_expr);
    let snake_case_name = var_name.to_snake_case();

    if var_name == snake_case_name {
        return None;
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
    .with_primary_label("non-snake_case name");

    let fix_data = SnakeCaseFixData {
        var_name_span: name_expr.span,
        usage_spans: find_variable_usages(*var_id, ctx),
    };

    Some((violation, fix_data))
}

struct SnakeCaseVariables;

impl DetectFix for SnakeCaseVariables {
    type FixInput<'a> = SnakeCaseFixData;

    fn id(&self) -> &'static str {
        "snake_case_variables"
    }

    fn short_description(&self) -> &'static str {
        "Variables should use snake_case naming convention"
    }

    fn source_link(&self) -> Option<&'static str> {
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

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let var_name = context.span_text(fix_data.var_name_span);
        let snake_case_name = var_name.to_snake_case();
        let usage_replacement = format!("${snake_case_name}");
        let replacements = std::iter::once(Replacement::new(
            fix_data.var_name_span,
            snake_case_name.clone(),
        ))
        .chain(
            fix_data
                .usage_spans
                .iter()
                .map(|usage_span| Replacement::new(*usage_span, usage_replacement.clone())),
        )
        .collect();
        Some(Fix {
            explanation: format!("Rename variable '{var_name}' to '{snake_case_name}'").into(),
            replacements,
        })
    }
}

pub static RULE: &dyn Rule = &SnakeCaseVariables;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
