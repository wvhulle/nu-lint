pub mod curl;
pub mod eza;
pub mod fd;
pub mod hostname;
pub mod jq;
pub mod printenv;
pub mod rg;
pub mod unnecessary_hat;
pub mod wget;
pub mod which;

use super::sets::RuleSet;
use crate::rule::Rule;

pub const RULES: &[Rule] = &[
    curl::rule(),
    eza::rule(),
    fd::rule(),
    hostname::rule(),
    jq::rule(),
    printenv::rule(),
    rg::rule(),
    unnecessary_hat::rule(),
    wget::rule(),
    which::rule(),
];

pub const fn rule_set() -> RuleSet {
    RuleSet {
        name: "external-tools",
        explanation: "Replace modern CLI tools with native Nushell equivalents",
        rules: RULES,
    }
}
