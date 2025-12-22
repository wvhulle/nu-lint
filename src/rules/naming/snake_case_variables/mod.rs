use heck::ToSnakeCase;
use nu_protocol::{
    Span, VarId,
    ast::{Argument, Call, Expr},
};

use crate::{
    LintLevel,
    ast::span::SpanExt,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

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

fn check_call(call: &Call, ctx: &LintContext) -> Option<Violation> {
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

    // Create replacements for declaration and all usages
    let mut replacements = vec![Replacement {
        span: name_expr.span.into(),
        replacement_text: snake_case_name.clone().into(),
    }];

    for usage_span in find_variable_usages(*var_id, ctx) {
        if usage_span.source_code(ctx).starts_with('$') {
            replacements.push(Replacement {
                span: usage_span.into(),
                replacement_text: format!("${snake_case_name}").into(),
            });
        }
    }

    let var_type = if is_mutable {
        "Mutable variable"
    } else {
        "Variable"
    };

    Some(
        Violation::new(
            format!("{var_type} '{var_name}' should use snake_case naming convention"),
            name_expr.span,
        )
        .with_primary_label("non-snake_case name")
        .with_help(format!("Consider renaming to: {snake_case_name}"))
        .with_fix(Fix::with_explanation(
            format!("Rename variable '{var_name}' to '{snake_case_name}'"),
            replacements,
        )),
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        check_call(call, ctx).into_iter().collect()
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "snake_case_variables",
        "Variables should use snake_case naming convention",
        check,
        LintLevel::Warning,
    )
    .with_doc_url("https://www.nushell.sh/book/style_guide.html#variables-and-command-parameters")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
