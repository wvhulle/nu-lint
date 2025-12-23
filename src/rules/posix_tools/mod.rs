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
