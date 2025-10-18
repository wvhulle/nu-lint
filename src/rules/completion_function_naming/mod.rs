use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Get all custom function definitions
    let functions = context.new_user_functions();

    for (_decl_id, decl) in functions {
        let func_name = &decl.signature().name;

        // Check if the function name suggests it's a completion function
        // but doesn't follow the nu-complete pattern
        let name_lower = func_name.to_lowercase();

        // Heuristics for completion functions:
        // - Contains "complete" or "completion"
        // - Used in completions context (we'd need to check usage)
        if (name_lower.contains("complete") || name_lower.contains("completion"))
            && !func_name.starts_with("nu-complete ")
        {
            let span = context.find_declaration_span(func_name);

            violations.push(Violation {
                rule_id: "completion_function_naming".into(),
                severity: Severity::Warning,
                message: format!(
                    "Completion function '{func_name}' should use 'nu-complete' prefix"
                ).into(),
                span,
                suggestion: Some(format!(
                    "Consider renaming to: nu-complete {}",
                    func_name
                        .replace("complete", "")
                        .replace("completion", "")
                        .trim()
                ).into()),
                fix: None,
                file: None,
            });
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "completion_function_naming",
        RuleCategory::Naming,
        Severity::Warning,
        "Completion functions should use 'nu-complete' prefix for clarity",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
