#[cfg(test)]
use std::borrow::Cow;
use std::hash::{Hash, Hasher};

use crate::{context::LintContext, violation::Violation};

/// A concrete rule struct that wraps the check function
#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub id: &'static str,
    pub explanation: &'static str,
    pub doc_url: Option<&'static str>,
    pub(crate) check: fn(&LintContext) -> Vec<Violation>,
}

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Rule {}

impl Rule {
    pub(crate) const fn new(
        id: &'static str,
        explanation: &'static str,
        check: fn(&LintContext) -> Vec<Violation>,
    ) -> Self {
        Self {
            id,
            explanation,
            doc_url: None,
            check,
        }
    }

    #[must_use]
    pub const fn with_doc_url(mut self, url: &'static str) -> Self {
        self.doc_url = Some(url);
        self
    }
}

#[cfg(test)]
impl Rule {
    fn run_check(&self, code: &str) -> Vec<Violation> {
        LintContext::test_with_parsed_source(code, |context| (self.check)(&context))
    }

    fn first_violation(&self, code: &str) -> Violation {
        let violations = self.run_check(code);
        assert!(
            !violations.is_empty(),
            "Expected rule '{}' to detect violations, but found none",
            self.id
        );
        violations.into_iter().next().unwrap()
    }

    fn first_replacement_text(&self, code: &str) -> Cow<'static, str> {
        let fix = self
            .first_violation(code)
            .fix
            .expect("Expected violation to have a fix");
        assert!(
            !fix.replacements.is_empty(),
            "Expected fix to have replacements"
        );
        fix.replacements
            .into_iter()
            .next()
            .unwrap()
            .replacement_text
    }

    #[track_caller]
    pub fn assert_detects(&self, code: &str) {
        let violations = self.run_check(code);
        assert!(
            !violations.is_empty(),
            "Expected rule '{}' to detect violations in code, but found none",
            self.id
        );
    }

    #[track_caller]
    pub fn assert_ignores(&self, code: &str) {
        let violations = self.run_check(code);
        assert!(
            violations.is_empty(),
            "Expected rule '{}' to ignore code, but found {} violations",
            self.id,
            violations.len()
        );
    }

    #[track_caller]
    pub fn assert_count(&self, code: &str, expected: usize) {
        let violations = self.run_check(code);
        assert_eq!(
            violations.len(),
            expected,
            "Expected rule '{}' to find exactly {} violation(s), but found {}",
            self.id,
            expected,
            violations.len()
        );
    }

    #[track_caller]
    pub fn assert_replacement_contains(&self, code: &str, expected_text: &str) {
        let replacement_text = self.first_replacement_text(code);
        assert!(
            replacement_text.contains(expected_text),
            "Expected fix replacement text to contain '{expected_text}', but got: \
             {replacement_text}"
        );
    }

    #[track_caller]
    pub fn assert_replacement_is(&self, bad_code: &str, expected_code: &str) {
        let replacement_text = self.first_replacement_text(bad_code);
        assert_eq!(
            replacement_text.as_ref(),
            expected_code,
            "Expected fix to produce exact code"
        );
    }

    #[track_caller]
    pub fn assert_fix_explanation_contains(&self, code: &str, expected_text: &str) {
        let fix = self
            .first_violation(code)
            .fix
            .expect("Expected violation to have a fix");
        assert!(
            fix.explanation.contains(expected_text),
            "Expected fix explanation to contain '{expected_text}', but got: {}",
            fix.explanation
        );
    }

    #[track_caller]
    pub fn assert_help_contains(&self, code: &str, expected_text: &str) {
        let help = self
            .first_violation(code)
            .help
            .expect("Expected violation to have help text");
        assert!(
            help.contains(expected_text),
            "Expected help to contain '{expected_text}', but got: {help}"
        );
    }

    #[allow(unused, reason = "Will be used.")]
    #[track_caller]
    pub fn assert_span_label_contains(&self, code: &str, expected_text: &str) {
        let violation = self.first_violation(code);
        let label_texts: Vec<&str> = violation
            .extra_labels
            .iter()
            .filter_map(|l| l.label())
            .collect();

        assert!(
            violation
                .primary_label
                .is_some_and(|text| text.contains(expected_text)),
            "Expected a label to contain '{expected_text}', but got labels: {label_texts:?}"
        );
    }

    #[track_caller]
    pub fn assert_labels_contain(&self, code: &str, expected_text: &str) {
        let violation = self.first_violation(code);
        let label_texts: Vec<&str> = violation
            .extra_labels
            .iter()
            .filter_map(|l| l.label())
            .collect();

        assert!(
            label_texts.iter().any(|t| t.contains(expected_text)),
            "Expected a label to contain '{expected_text}', but got labels: {label_texts:?}"
        );
    }

    #[track_caller]
    pub fn assert_replacement_erases(&self, code: &str, erased_text: &str) {
        let fix = self
            .first_violation(code)
            .fix
            .expect("Expected violation to have a fix");
        assert!(
            !fix.replacements.is_empty(),
            "Expected fix to have replacements"
        );

        let replacement = &fix.replacements[0];
        let original_text = &code[replacement.span.start..replacement.span.end];
        let replacement_text = &replacement.replacement_text;

        assert!(
            original_text.contains(erased_text),
            "Original text should contain '{erased_text}', but got: {original_text}"
        );
        assert!(
            !replacement_text.contains(erased_text),
            "Expected replacement text to not contain '{erased_text}', but it still appears in: \
             {replacement_text}"
        );
    }
}
