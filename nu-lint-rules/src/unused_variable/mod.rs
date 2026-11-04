use std::collections::{HashMap, HashSet};

use nu_lint_analysis::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, DiagnosticTag, Fix, Replacement},
};
use nu_protocol::{
    Category, Span, VarId,
    ast::{Call, Expr, Expression, Traverse},
};

struct FixData {
    var_name_span: Span,
    declaration_span: Span,
    value_is_removable: bool,
}

fn builtin_is_side_effect_free(call: &Call, context: &LintContext) -> bool {
    let decl = context.working_set.get_decl(call.decl_id);
    if !decl.is_builtin() {
        return false;
    }
    match decl.signature().category {
        Category::Bytes
        | Category::Conversions
        | Category::Date
        | Category::Filters
        | Category::Formats
        | Category::Generators
        | Category::Hash
        | Category::Math
        | Category::Path
        | Category::Random => true,
        Category::Strings => call.builtin_name(context) != Some("print"),
        Category::Core => matches!(
            call.builtin_name(context),
            Some("do" | "if" | "let" | "match" | "mut")
        ),
        _ => false,
    }
}

fn value_is_side_effect_free(value: &Expression, context: &LintContext) -> bool {
    let mut effectful_expressions: Vec<Span> = Vec::new();
    value.flat_map(
        context.working_set,
        &|nested_expr| match &nested_expr.expr {
            Expr::ExternalCall(..) => vec![nested_expr.span],
            Expr::Call(call) if !builtin_is_side_effect_free(call, context) => {
                vec![nested_expr.span]
            }
            _ => vec![],
        },
        &mut effectful_expressions,
    );
    effectful_expressions.is_empty()
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
        let mut declarations: Vec<(VarId, Span, Span, bool)> = Vec::new();
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
                let value_is_removable = call
                    .get_positional_arg(1)
                    .is_some_and(|value| value_is_side_effect_free(value, context));
                vec![(var_id, var_name_span, expr.span, value_is_removable)]
            },
            &mut declarations,
        );

        let decl_map: HashMap<VarId, (Span, Span, bool)> = declarations
            .into_iter()
            .map(|(id, name_span, span, value_is_removable)| {
                (id, (name_span, span, value_is_removable))
            })
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
            .map(|(_, (var_name_span, span, value_is_removable))| {
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
                        value_is_removable,
                    },
                )
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        if !fix_data.value_is_removable {
            return None;
        }

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
