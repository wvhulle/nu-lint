use heck::ToSnakeCase;
use nu_protocol::ast::{Argument, Expr};

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Check if a variable name follows `snake_case` convention
fn is_valid_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Allow single characters
    if name.len() == 1 {
        return name.chars().all(|c| c.is_ascii_lowercase() || c == '_');
    }

    // Must start with lowercase letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_lowercase() && first_char != '_' {
        return false;
    }

    // Check snake_case pattern: lowercase letters, numbers, and underscores
    // Cannot have consecutive underscores
    let chars: Vec<char> = name.chars().collect();
    chars.windows(2).all(|w| {
        let (current, next) = (w[0], w[1]);
        // All characters must be valid
        let valid_char = matches!(current, 'a'..='z' | '0'..='9' | '_');
        // No consecutive underscores
        let no_double_underscore = !(current == '_' && next == '_');
        valid_char && no_double_underscore
    }) && matches!(chars.last(), Some('a'..='z' | '0'..='9' | '_'))
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        match &expr.expr {
            Expr::Call(call) => {
                // Check for let/mut assignments in command calls
                let decl = ctx.working_set.get_decl(call.decl_id);
                let (is_mutable, should_check) = match decl.name() {
                    "let" => (false, true),
                    "mut" => (true, true),
                    _ => (false, false),
                };

                if should_check {
                    // The first argument to let/mut should be the variable name
                    if let Some(Argument::Positional(name_expr)) = call.arguments.first() {
                        let var_name = ctx
                            .source
                            .get(name_expr.span.start..name_expr.span.end)
                            .unwrap_or("");

                        if !is_valid_snake_case(var_name) {
                            let var_type = if is_mutable {
                                "Mutable variable"
                            } else {
                                "Variable"
                            };
                            let snake_case_name = var_name.to_snake_case();
                            let fix = Some(Fix {
                                description: format!(
                                    "Rename variable '{var_name}' to '{snake_case_name}'"
                                )
                                .into(),
                                replacements: vec![Replacement {
                                    span: name_expr.span,
                                    new_text: snake_case_name.clone().into(),
                                }],
                            });

                            let mut violation = RuleViolation::new_dynamic(
                                "snake_case_variables",
                                format!(
                                    "{var_type} '{var_name}' should use snake_case naming \
                                     convention"
                                ),
                                name_expr.span,
                            )
                            .with_suggestion_dynamic(format!(
                                "Consider renaming to: {snake_case_name}"
                            ));

                            if let Some(f) = fix {
                                violation = violation.with_fix(f);
                            }

                            return vec![violation];
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "snake_case_variables",
        RuleCategory::Naming,
        Severity::Info,
        "Variables should use snake_case naming convention",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
