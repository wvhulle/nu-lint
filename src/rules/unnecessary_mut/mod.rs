use std::collections::{HashMap, HashSet};

use nu_protocol::{
    Span, VarId,
    ast::{Expr, Expression},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

/// Find the span of 'mut ' keyword before the variable name
/// Returns a global span (will be normalized later by the engine)
fn find_mut_keyword_span(context: &LintContext, var_span: Span) -> Span {
    let text_before = context.source_before_span(var_span);
    let search_text = if text_before.len() > 20 {
        &text_before[text_before.len() - 20..]
    } else {
        text_before
    };

    if let Some(mut_pos) = search_text.rfind("mut ") {
        // Calculate global position
        let offset_in_search = search_text.len() - mut_pos;
        let abs_mut_start = var_span.start.saturating_sub(offset_in_search);
        let abs_mut_end = abs_mut_start + 4;
        return Span::new(abs_mut_start, abs_mut_end);
    }

    var_span
}

fn extract_mut_declaration(
    expr: &Expression,
    context: &LintContext,
) -> Option<(VarId, String, Span, Span)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    let decl_name = call.get_call_name(context);
    if decl_name != "mut" {
        return None;
    }

    let (var_id, var_name, var_span) = call.extract_variable_declaration(context)?;

    if var_name.starts_with('_') {
        return None;
    }

    let mut_span = find_mut_keyword_span(context, var_span);
    Some((var_id, var_name, var_span, mut_span))
}

fn check(context: &LintContext) -> Vec<Violation> {
    use nu_protocol::ast::Traverse;

    let mut mut_declarations: Vec<(VarId, String, Span, Span)> = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| extract_mut_declaration(expr, context).into_iter().collect(),
        &mut mut_declarations,
    );

    let mut_variables: HashMap<VarId, (String, Span, Span)> = mut_declarations
        .into_iter()
        .map(|(id, name, decl_span, mut_span)| (id, (name, decl_span, mut_span)))
        .collect();

    let mut reassigned: Vec<VarId> = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| expr.extract_assigned_variable().into_iter().collect(),
        &mut reassigned,
    );

    let reassigned_vars: HashSet<VarId> = reassigned.into_iter().collect();

    // Generate violations for mut variables that were never reassigned
    let mut violations = Vec::new();
    for (var_id, (var_name, decl_span, mut_span)) in mut_variables {
        if !reassigned_vars.contains(&var_id) {
            let fix = Fix::with_explanation(
                format!("Remove 'mut' keyword from variable '{var_name}'"),
                vec![Replacement::new(mut_span, "")],
            );

            violations.push(
                Violation::new(
                    format!("Variable '{var_name}' is declared as 'mut' but never reassigned"),
                    mut_span,
                )
                .with_primary_label("unnecessary mut keyword")
                .with_extra_label("variable never reassigned", decl_span)
                .with_help(format!("Remove 'mut' keyword:\nlet {var_name} = ..."))
                .with_fix(fix),
            );
        }
    }

    violations
}

pub const RULE: Rule = Rule::new(
    "unnecessary_mut",
    "Variables should only be marked 'mut' when they are actually reassigned",
    check,
    LintLevel::Warning,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/book/variables.html#mutable-variables");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
