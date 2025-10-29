use std::collections::{HashMap, HashSet};

use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use nu_protocol::ast::Traverse;

    let mut violations = Vec::new();
    let mut variable_declarations: Vec<(VarId, Span)> = Vec::new();
    #[allow(clippy::type_complexity)]
    let mut closure_contexts: Vec<(Span, Vec<(VarId, Span, String)>)> = Vec::new();

    // First pass: collect all variable declarations
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr {
                let decl_name = context.working_set.get_decl(call.decl_id).name();
                if (decl_name == "let" || decl_name == "mut")
                    && let Some(var_arg) = call.arguments.first()
                {
                    let (nu_protocol::ast::Argument::Positional(var_expr)
                    | nu_protocol::ast::Argument::Unknown(var_expr)) = var_arg
                    else {
                        return vec![];
                    };

                    if let Expr::VarDecl(var_id) = &var_expr.expr {
                        return vec![(*var_id, var_expr.span)];
                    }
                }
            }
            vec![]
        },
        &mut variable_declarations,
    );

    // Second pass: find closures and analyze variable usage within them
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            match &expr.expr {
                Expr::Closure(block_id) => {
                    let block = context.working_set.get_block(*block_id);
                    let mut closure_vars = Vec::new();

                    // Traverse the closure body to find variable references
                    for pipeline in &block.pipelines {
                        for element in &pipeline.elements {
                            element.expr.flat_map(
                                context.working_set,
                                &|inner_expr| {
                                    if let Expr::Var(var_id) = &inner_expr.expr {
                                        let var_name = &context.source
                                            [inner_expr.span.start..inner_expr.span.end];
                                        return vec![(
                                            *var_id,
                                            inner_expr.span,
                                            var_name.to_string(),
                                        )];
                                    }
                                    vec![]
                                },
                                &mut closure_vars,
                            );
                        }
                    }

                    vec![(expr.span, closure_vars)]
                }
                _ => vec![],
            }
        },
        &mut closure_contexts,
    );

    // Convert declarations to HashMap for quick lookup
    let variable_scopes: HashMap<VarId, Span> = variable_declarations.into_iter().collect();

    // Analyze each closure context
    for (closure_span, closure_vars) in closure_contexts {
        // Get built-in variable IDs that are safe to use in closures
        let builtin_vars: HashSet<String> = ["$in", "$it", "$env"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        for (var_id, use_span, var_name) in closure_vars {
            // Skip built-in variables that are safe in closures
            if builtin_vars.contains(&var_name) || var_name.starts_with("$env.") {
                continue;
            }

            // Check if this variable was declared outside the closure
            if let Some(decl_span) = variable_scopes.get(&var_id) {
                // Check if the declaration is outside the closure
                if decl_span.start < closure_span.start || decl_span.end > closure_span.end {
                    violations.push(
                            RuleViolation::new_dynamic(
                                "closure_variable_capture",
                                format!("Variable '{var_name}' captured in closure may be stale or out of scope"),
                                use_span,
                            )
                            .with_suggestion_dynamic(
                                format!("Consider using '$in' parameter or passing '{var_name}' as closure parameter"),
                            ),
                        );
                }
            }
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "closure_variable_capture",
        RuleCategory::ErrorHandling,
        Severity::Error,
        "Detect potentially problematic variable captures in closures",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
