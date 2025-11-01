use heck::{ToKebabCase, ToSnakeCase};
use nu_protocol::Span;

use crate::violation::{Fix, Replacement, RuleViolation};

pub mod kebab_case_commands;

pub mod completion_function_naming;
pub mod screaming_snake_constants;
pub mod snake_case_variables;

/// Extension trait for string naming convention validation
pub trait NuNaming {
    /// Check if a name is valid kebab-case
    fn is_valid_kebab_case(&self) -> bool;

    /// Check if a name is valid `snake_case`
    fn is_valid_snake_case(&self) -> bool;

    /// Create a naming convention violation with fix
    fn create_naming_violation(
        &self,
        rule_id: &'static str,
        item_type: &str,
        suggested_name: &str,
        name_span: Span,
    ) -> RuleViolation;
}

impl NuNaming for str {
    fn is_valid_kebab_case(&self) -> bool {
        self == self.to_kebab_case()
    }

    fn is_valid_snake_case(&self) -> bool {
        self == self.to_snake_case()
    }

    fn create_naming_violation(
        &self,
        rule_id: &'static str,
        item_type: &str,
        suggested_name: &str,
        name_span: Span,
    ) -> RuleViolation {
        let fix = Fix {
            description: format!("Rename {item_type} '{self}' to '{suggested_name}'").into(),
            replacements: vec![Replacement {
                span: name_span,
                new_text: suggested_name.to_string().into(),
            }],
        };

        RuleViolation::new_dynamic(
            rule_id,
            format!("{item_type} '{self}' should follow naming convention"),
            name_span,
        )
        .with_suggestion_dynamic(format!("Consider renaming to: {suggested_name}"))
        .with_fix(fix)
    }
}
