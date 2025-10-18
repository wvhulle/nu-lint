use heck::ToSnakeCase;

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, Severity, Violation},
    rule::{Rule, RuleCategory},
    visitor::{AstVisitor, VisitContext},
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

    // Check snake_case pattern: lowercase letters, numbers, and underscores
    // Must start with lowercase letter or underscore
    // Cannot have consecutive underscores
    name.chars().enumerate().all(|(i, c)| {
        match c {
            'a'..='z' | '0'..='9' => true,
            '_' => {
                // First character can be underscore
                if i == 0 {
                    return true;
                }
                // Cannot have consecutive underscores
                name.chars().nth(i + 1) != Some('_')
            }
            _ => false,
        }
    }) && name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut visitor = SnakeCaseVariablesVisitor::new();
    context.walk_ast(&mut visitor);
    visitor.violations
}

pub fn rule() -> Rule {
    Rule::new(
        "snake_case_variables",
        RuleCategory::Naming,
        Severity::Warning,
        "Variables should use snake_case naming convention",
        check,
    )
}

/// AST visitor that checks variable naming using AST traversal
struct SnakeCaseVariablesVisitor {
    violations: Vec<Violation>,
}

impl SnakeCaseVariablesVisitor {
    fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    fn check_variable_name(&mut self, var_name: &str, span: nu_protocol::Span, is_mutable: bool) {
        if !is_valid_snake_case(var_name) {
            let var_type = if is_mutable {
                "Mutable variable"
            } else {
                "Variable"
            };
            let snake_case_name = var_name.to_snake_case();

            let fix = Some(Fix {
                description: format!("Rename variable '{var_name}' to '{snake_case_name}'").into(),
                replacements: vec![Replacement {
                    span,
                    new_text: snake_case_name.clone().into(),
                }],
            });

            self.violations.push(Violation {
                rule_id: "snake_case_variables".into(),
                severity: Severity::Warning,
                message: format!("{var_type} '{var_name}' should use snake_case naming convention").into(),
                span,
                suggestion: Some(format!("Consider renaming to: {snake_case_name}").into()),
                fix,
                file: None,
            });
        }
    }
}

impl AstVisitor for SnakeCaseVariablesVisitor {
    fn visit_call(&mut self, call: &nu_protocol::ast::Call, context: &VisitContext) {
        // Check for let/mut assignments in command calls
        let decl = context.get_decl(call.decl_id);
        match decl.name() {
            "let" => {
                // The first argument to let should be the variable name
                if let Some(first_arg) = call.arguments.first()
                    && let nu_protocol::ast::Argument::Positional(expr) = first_arg
                {
                    // Extract variable name from the span
                    let var_name = context.get_span_contents(expr.span);
                    self.check_variable_name(var_name, expr.span, false);
                }
            }
            "mut" => {
                // The first argument to mut should be the variable name
                if let Some(first_arg) = call.arguments.first()
                    && let nu_protocol::ast::Argument::Positional(expr) = first_arg
                {
                    // Extract variable name from the span
                    let var_name = context.get_span_contents(expr.span);
                    self.check_variable_name(var_name, expr.span, true);
                }
            }
            _ => {}
        }

        // Continue walking
        crate::visitor::walk_call(self, call, context);
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
