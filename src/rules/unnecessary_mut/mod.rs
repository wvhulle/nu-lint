use std::collections::{HashMap, HashSet};

use nu_protocol::{
    Span, VarId,
    ast::{Expr, Expression},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct UnnecessaryMutFixData {
    var_name: String,
    mut_span: Span,
}

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

struct UnnecessaryMut;

impl DetectFix for UnnecessaryMut {
    type FixInput = UnnecessaryMutFixData;

    fn id(&self) -> &'static str {
        "unnecessary_mut"
    }

    fn explanation(&self) -> &'static str {
        "Variables should only be marked 'mut' when they are actually reassigned"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/variables.html#mutable-variables")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
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
                let violation = Detection::from_global_span(
                    format!("Variable '{var_name}' is declared as 'mut' but never reassigned"),
                    mut_span,
                )
                .with_primary_label("unnecessary mut keyword")
                .with_extra_label("variable never reassigned", decl_span)
                .with_help(format!("Remove 'mut' keyword:\nlet {var_name} = ..."));

                let fix_data = UnnecessaryMutFixData { var_name, mut_span };

                violations.push((violation, fix_data));
            }
        }

        violations
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        Some(Fix::with_explanation(
            format!("Remove 'mut' keyword from variable '{}'", fix_data.var_name),
            vec![Replacement::new(fix_data.mut_span, "")],
        ))
    }
}

pub static RULE: &dyn Rule = &UnnecessaryMut;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
