use core::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::LintLevel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintSet {
    pub name: String,
    pub rules: HashMap<String, Option<LintLevel>>,
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
