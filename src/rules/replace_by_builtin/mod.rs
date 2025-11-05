// Individual rules for prefer_builtin_* commands
//
// Each common command has its own rule for better maintainability.
// Less common commands are grouped in the 'other' subrule.
pub mod cat;
pub mod echo;
pub mod find;
pub mod grep;
pub mod head;
pub mod http;
pub mod jq;
pub mod ls;
pub mod other;
pub mod path;
pub mod sed;
pub mod sort;
pub mod tail;
pub mod uniq;
