#[cfg(test)]
use std::borrow::Cow;
use std::{
    any::TypeId,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
};

use crate::{
    Fix, LintLevel,
    context::LintContext,
    violation::{Detection, Violation},
};

/// Trait for implementing lint rules with typed fix data.
///
/// # Example
/// ```ignore
/// struct MyRule;
/// impl DetectFix for MyRule {
///     type FixInput = ();
///     fn id(&self) -> &'static str { "my_rule" }
///     fn explanation(&self) -> &'static str { "Checks for..." }
///     fn level(&self) -> LintLevel { LintLevel::Warning }
///     fn detect(&self, ctx: &LintContext) -> Vec<(Detection, ())> { vec![] }
/// }
/// pub static RULE: &dyn Rule = &MyRule;
/// ```
pub trait DetectFix: Send + Sync + 'static {
    type FixInput: Send + Sync;

    fn id(&self) -> &'static str;
    fn explanation(&self) -> &'static str;
    fn doc_url(&self) -> Option<&'static str> {
        None
    }
    fn level(&self) -> LintLevel;
    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)>;
    fn fix(&self, _context: &LintContext, _fix_data: &Self::FixInput) -> Option<Fix> {
        None
    }

    /// Pairs violations with default fix input (for rules with `FixInput =
    /// ()`).
    fn no_fix(violations: Vec<Detection>) -> Vec<(Detection, Self::FixInput)>
    where
        Self::FixInput: Default,
    {
        violations
            .into_iter()
            .map(|v| (v, Self::FixInput::default()))
            .collect()
    }
}

/// Type-erased interface for storing and executing rules.
///
/// All `DetectFix` implementations automatically implement this via a blanket
/// impl.
pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn explanation(&self) -> &'static str;
    fn doc_url(&self) -> Option<&'static str>;
    fn level(&self) -> LintLevel;
    fn has_auto_fix(&self) -> bool;
    fn check(&self, context: &LintContext) -> Vec<Violation>;
}

impl<T: DetectFix> Rule for T {
    fn id(&self) -> &'static str {
        DetectFix::id(self)
    }

    fn explanation(&self) -> &'static str {
        DetectFix::explanation(self)
    }

    fn doc_url(&self) -> Option<&'static str> {
        DetectFix::doc_url(self)
    }

    fn level(&self) -> LintLevel {
        DetectFix::level(self)
    }

    fn has_auto_fix(&self) -> bool {
        TypeId::of::<T::FixInput>() != TypeId::of::<()>()
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        self.detect(context)
            .into_iter()
            .map(|(detected, fix_data)| {
                let fix = self.fix(context, &fix_data);
                Violation::from_detected(detected, fix)
            })
            .collect()
    }
}

impl Debug for dyn Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Rule")
            .field("id", &self.id())
            .field("level", &self.level())
            .field("has_auto_fix", &self.has_auto_fix())
            .finish()
    }
}

impl Hash for dyn Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl PartialEq for dyn Rule {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for dyn Rule {}

#[cfg(test)]
impl dyn Rule {
    fn run_check(&self, code: &str) -> Vec<Violation> {
        LintContext::test_get_violations(code, |context| self.check(context))
    }

    fn first_violation(&self, code: &str) -> Violation {
        let violations = self.run_check(code);
        assert!(
            !violations.is_empty(),
            "Expected rule '{}' to detect violations, but found none",
            self.id()
        );
        violations.into_iter().next().unwrap()
    }

    pub fn first_replacement_text(&self, code: &str) -> Cow<'static, str> {
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
            self.id()
        );
    }

    #[track_caller]
    pub fn assert_ignores(&self, code: &str) {
        let violations = self.run_check(code);
        assert!(
            violations.is_empty(),
            "Expected rule '{}' to ignore code, but found {} violations",
            self.id(),
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
            self.id(),
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

    #[track_caller]
    pub fn assert_labels_contain(&self, code: &str, expected_text: &str) {
        let violation = self.first_violation(code);
        let label_texts: Vec<&str> = violation
            .extra_labels
            .iter()
            .filter_map(|(_, label)| label.as_deref())
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
        let file_span = replacement.file_span();
        let original_text = &code[file_span.start..file_span.end];
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
