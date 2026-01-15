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
pub trait DetectFix: Send + Sync + 'static {
    /// Data used to construct a fix (optional)
    type FixInput<'a>: Send + Sync;

    /// Should only contain lower-case letters and underscores. Imperative,
    /// descriptive and around 2-4 keywords.
    fn id(&self) -> &'static str;

    /// Default lint level of rule
    fn level(&self) -> Option<LintLevel>;

    /// Create a vector of detections of violations of the rule
    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)>;

    /// Description shown next to rule ID in table
    fn short_description(&self) -> &'static str;

    /// Optional long description formatted as additional help next to
    /// violations. Remove if too short and use `short_description` instead.
    fn long_description(&self) -> Option<&'static str> {
        None
    }

    /// Optional hyperlink to (semi-)official Nu shell documentation or style
    /// guide
    fn source_link(&self) -> Option<&'static str> {
        None
    }

    /// Rules that conflict with this rule. When both rules are enabled,
    /// the linter will error at startup.
    fn conflicts_with(&self) -> &'static [&'static dyn Rule] {
        &[]
    }

    fn fix(&self, _context: &LintContext, _fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        None
    }

    /// Pairs violations with default fix input (for rules with `FixInput =
    /// ()`).
    fn no_fix<'a>(detections: Vec<Detection>) -> Vec<(Detection, Self::FixInput<'a>)>
    where
        Self::FixInput<'a>: Default,
    {
        detections
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
    fn short_description(&self) -> &'static str;
    fn source_link(&self) -> Option<&'static str>;
    fn level(&self) -> Option<LintLevel>;
    fn has_auto_fix(&self) -> bool;
    fn conflicts_with(&self) -> &'static [&'static dyn Rule];
    fn check(&self, context: &LintContext) -> Vec<Violation>;
}

impl<T: DetectFix> Rule for T {
    fn id(&self) -> &'static str {
        DetectFix::id(self)
    }

    fn short_description(&self) -> &'static str {
        DetectFix::short_description(self)
    }

    fn source_link(&self) -> Option<&'static str> {
        DetectFix::source_link(self)
    }

    fn level(&self) -> Option<LintLevel> {
        DetectFix::level(self)
    }

    fn has_auto_fix(&self) -> bool {
        TypeId::of::<T::FixInput<'static>>() != TypeId::of::<()>()
    }

    fn conflicts_with(&self) -> &'static [&'static dyn Rule] {
        DetectFix::conflicts_with(self)
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        self.detect(context)
            .into_iter()
            .map(|(detected, fix_data)| {
                let long_description = self.long_description();
                let fix = self.fix(context, &fix_data);
                Violation::from_detected(detected, fix, long_description)
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

    #[track_caller]
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

    /// Assumes there is only one violation and fix in the code (with zero or
    /// more replacements)
    pub fn apply_first_fix(&self, code: &str) -> String {
        let violation = self.first_violation(code);
        let fix = violation.fix.expect("Expected violation to have a fix");
        assert!(
            !fix.replacements.is_empty(),
            "Expected fix to have replacements"
        );

        let mut replacements = fix.replacements;
        replacements.sort_by(|a, b| b.file_span().start.cmp(&a.file_span().start));

        let mut result = code.to_string();
        for replacement in replacements {
            let start = replacement.file_span().start;
            let end = replacement.file_span().end;
            result.replace_range(start..end, &replacement.replacement_text);
        }
        result
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
    pub fn assert_fixed_contains(&self, code: &str, expected_text: &str) {
        let fixed = self.apply_first_fix(code);
        assert!(
            fixed.contains(expected_text),
            "Expected fixed code to contain `{expected_text}`, but it didn't, it was `{fixed}`"
        );
    }

    #[track_caller]
    pub fn assert_fixed_is(&self, bad_code: &str, expected_code: &str) {
        let fixed = self.apply_first_fix(bad_code);
        assert!(
            fixed == expected_code,
            "Expected fix to be `{fixed}` but received `{expected_code}`"
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
    pub fn assert_fix_erases(&self, code: &str, erased_text: &str) {
        let fixed = self.apply_first_fix(code);
        assert!(
            code.contains(erased_text),
            "Original code should contain '{erased_text}', but it doesn't"
        );
        assert!(
            !fixed.contains(erased_text),
            "Expected fixed code to not contain '{erased_text}', but it still appears in: {fixed}"
        );
    }
}
