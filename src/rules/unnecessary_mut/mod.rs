use std::collections::{HashMap, HashSet};

use lsp_types::DiagnosticTag;
use nu_protocol::{
    Span, VarId,
    ast::{Expr, Expression, Traverse},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct UnnecessaryMutFixData {
    var_id: VarId,
    var_name: String,
    var_span: Span,
    keyword_span: Span,
    mut_to_remove: Span,
}

fn extract_mut_declaration(
    expr: &Expression,
    context: &LintContext,
) -> Option<UnnecessaryMutFixData> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if !call.is_call_to_command("mut", context) {
        return None;
    }

    let (var_id, var_name, var_span) = call.extract_variable_declaration(context)?;

    if var_name.starts_with('_') {
        return None;
    }

    Some(UnnecessaryMutFixData {
        var_id,
        var_name,
        var_span,
        keyword_span: call.head,
        mut_to_remove: Span::new(call.head.start, var_span.start),
    })
}

struct UnnecessaryMut;

impl DetectFix for UnnecessaryMut {
    type FixInput<'a> = UnnecessaryMutFixData;

    fn id(&self) -> &'static str {
        "unnecessary_mut"
    }

    fn short_description(&self) -> &'static str {
        "Variable marked `mut` but never reassigned"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/variables.html#mutable-variables")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::UNNECESSARY]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut mut_declarations: Vec<UnnecessaryMutFixData> = Vec::new();

        context.ast.flat_map(
            context.working_set,
            &|expr| extract_mut_declaration(expr, context).into_iter().collect(),
            &mut mut_declarations,
        );

        let mut_variables: HashMap<VarId, UnnecessaryMutFixData> = mut_declarations
            .into_iter()
            .map(|data| (data.var_id, data))
            .collect();

        let mut reassigned: Vec<VarId> = Vec::new();

        context.ast.flat_map(
            context.working_set,
            &|expr| expr.extract_assigned_variable().into_iter().collect(),
            &mut reassigned,
        );

        let reassigned_vars: HashSet<VarId> = reassigned.into_iter().collect();

        mut_variables
            .into_iter()
            .filter(|(var_id, _)| !reassigned_vars.contains(var_id))
            .map(|(_, fix_data)| {
                let violation = Detection::from_global_span(
                    format!(
                        "Variable '{}' is declared as 'mut' but never reassigned",
                        fix_data.var_name
                    ),
                    fix_data.keyword_span,
                )
                .with_primary_label("unnecessary mut keyword")
                .with_extra_label("variable never reassigned", fix_data.var_span);

                (violation, fix_data)
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Remove 'mut' keyword from variable '{}'", fix_data.var_name)
                .into(),
            replacements: vec![Replacement::new(fix_data.mut_to_remove, "")],
        })
    }
}

pub static RULE: &dyn Rule = &UnnecessaryMut;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
