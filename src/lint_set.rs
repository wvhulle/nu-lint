use core::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
};
use std::{collections::HashMap, sync::LazyLock};

use serde::Serialize;

use crate::LintLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintSet {
    pub name: String,
    pub rules: HashMap<String, LintLevel>,
}

impl Serialize for LintSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Just serialize the name of the lint set
        serializer.serialize_str(&self.name)
    }
}

static BUILTIN_LINT_SETS: LazyLock<HashMap<&str, LintSet>> = LazyLock::new(|| {
    HashMap::from([
        (
            "naming",
            LintSet {
                name: "naming".to_string(),
                rules: HashMap::from([
                    ("snake_case_variables".to_string(), LintLevel::Deny),
                    ("kebab_case_commands".to_string(), LintLevel::Deny),
                    ("screaming_snake_constants".to_string(), LintLevel::Deny),
                ]),
            },
        ),
        (
            "idioms",
            LintSet {
                name: "idioms".to_string(),
                rules: HashMap::new(),
            },
        ),
        (
            "pedantic",
            LintSet {
                name: "pedantic".to_string(),
                rules: HashMap::new(),
            },
        ),
    ])
});

#[must_use]
pub fn builtin_lint_sets() -> &'static HashMap<&'static str, LintSet> {
    &BUILTIN_LINT_SETS
}

impl Hash for LintSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Display for LintSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
