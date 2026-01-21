use std::collections::{HashMap, HashSet};

use lsp_types::DiagnosticTag;
use nu_protocol::{
    Span, VarId,
    ast::{Expr, Traverse},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    var_name: String,
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

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::UNNECESSARY]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // 1. Collect all let/mut declarations
        let mut declarations: Vec<(VarId, String, Span)> = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| {
                let Expr::Call(call) = &expr.expr else {
                    return vec![];
                };
                let Some((var_id, var_name, _)) = call.extract_variable_declaration(context) else {
                    return vec![];
                };
                // Skip underscore-prefixed variables (intentionally unused)
                if var_name.starts_with('_') {
                    return vec![];
                }
                vec![(var_id, var_name, expr.span)]
            },
            &mut declarations,
        );

        let decl_map: HashMap<VarId, (String, Span)> = declarations
            .into_iter()
            .map(|(id, name, span)| (id, (name, span)))
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
            .map(|(_, (var_name, span))| {
                let detection = Detection::from_global_span(
                    format!("Variable '{var_name}' is declared but never used"),
                    span,
                )
                .with_primary_label("unused variable");

                (
                    detection,
                    FixData {
                        var_name,
                        declaration_span: span,
                    },
                )
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let removal_span = context.expand_span_to_full_lines(fix_data.declaration_span);

        Some(Fix {
            explanation: format!("Remove unused variable '{}'", fix_data.var_name).into(),
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
