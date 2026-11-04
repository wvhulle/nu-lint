use std::collections::{HashMap, HashSet};

use nu_lint_analysis::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, DiagnosticTag, Fix, Replacement},
};
use nu_protocol::{
    Span, VarId,
    ast::{Expr, Traverse},
};

struct FixData {
    var_name_span: Span,
    declaration_span: Span,
}

struct UnusedVariable;

impl DetectFix for UnusedVariable {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "unused_variable"
    }

    fn short_description(&self) -> &'static str {
        "Variable declared but never used"
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::Unnecessary]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // 1. Collect all let/mut declarations
        let mut declarations: Vec<(VarId, Span, Span)> = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| {
                let Expr::Call(call) = &expr.expr else {
                    return vec![];
                };
                let Some((var_id, var_name, var_name_span)) =
                    call.extract_variable_declaration(context)
                else {
                    return vec![];
                };
                // Skip underscore-prefixed variables (intentionally unused)
                if var_name.starts_with('_') {
                    return vec![];
                }
                vec![(var_id, var_name_span, expr.span)]
            },
            &mut declarations,
        );

        let decl_map: HashMap<VarId, (Span, Span)> = declarations
            .into_iter()
            .map(|(id, name_span, span)| (id, (name_span, span)))
            .collect();

        // 2. Collect all variable usages
        let mut usages: Vec<VarId> = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| expr.extract_direct_var().into_iter().collect(),
            &mut usages,
        );
        let used_vars: HashSet<VarId> = usages.into_iter().collect();

        // 3. Find unused declarations
        decl_map
            .into_iter()
            .filter(|(var_id, _)| !used_vars.contains(var_id))
            .map(|(_, (var_name_span, span))| {
                let detection = Detection::from_global_span(
                    format!(
                        "Variable '{}' is declared but never used",
                        context.span_text(var_name_span)
                    ),
                    span,
                )
                .with_primary_label("unused variable");

                (
                    detection,
                    FixData {
                        var_name_span,
                        declaration_span: span,
                    },
                )
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let removal_span = context.expand_span_to_statement(fix_data.declaration_span);

        Some(Fix {
            explanation: format!(
                "Remove unused variable '{}'",
                context.span_text(fix_data.var_name_span)
            )
            .into(),
            replacements: vec![Replacement::new(removal_span, String::new())],
        })
    }
}

pub static RULE: &dyn Rule = &UnusedVariable;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
