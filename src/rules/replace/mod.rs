//!  Each common command has its own rule for better maintainability.
//!  Less common commands are grouped in the 'other' subrule.
pub mod cat;
pub mod echo;
pub mod awk;
pub mod cut;
pub mod printenv;
pub mod date;
pub mod hostname;
pub mod man;
pub mod read;
pub mod wc;
pub mod which;
pub mod find;
pub mod grep;
pub mod head;
pub mod jq;
pub mod ls;
pub mod sed;
pub mod sort;
pub mod tail;
pub mod uniq;

pub mod curl;
pub mod wget;
pub mod fetch;
