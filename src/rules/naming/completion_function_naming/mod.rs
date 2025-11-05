use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn check(context: &LintContext) -> Vec<RuleViolation> {
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

            violations.push(
                RuleViolation::new_dynamic(
                    "completion_function_naming",
                    format!("Completion function '{func_name}' should use 'nu-complete' prefix"),
                    span,
                )
                .with_suggestion_dynamic(format!(
                    "Consider renaming to: nu-complete {}",
                    func_name
                        .replace("complete", "")
                        .replace("completion", "")
                        .trim()
                )),
            );
        }
    }

    violations
}

pub(crate) fn rule() -> Rule {
    Rule::new(
        "completion_function_naming",
        RuleCategory::Naming,
        Severity::Info,
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
