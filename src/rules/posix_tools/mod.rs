//!  Each common command has its own rule for better maintainability.
//!  Less common commands are grouped in the 'other' subrule.

pub mod awk;
pub mod cat;
pub mod cd;
pub mod cut;
pub mod date;
pub mod echo;
pub mod find;
pub mod grep;
pub mod head;
pub mod ls;

pub mod read;
pub mod sed;
pub mod sort;
pub mod tail;
pub mod uniq;
pub mod wc;

use super::sets::RuleSet;
use crate::rule::Rule;

pub const RULES: &[Rule] = &[
    awk::rule(),
    cat::rule(),
    cut::rule(),
    date::rule(),
    echo::rule(),
    find::rule(),
    grep::rule(),
    head::rule(),
    cd::rule(),
    ls::rule(),
    read::rule(),
    sed::rule(),
    sort::rule(),
    tail::rule(),
    uniq::rule(),
    wc::rule(),
];

pub const fn rule_set() -> RuleSet {
    RuleSet {
        name: "bashisms",
        explanation: "Replace common bash/POSIX commands with native Nushell equivalents",
        rules: RULES,
    }
}
