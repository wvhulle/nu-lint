//!  Each common command has its own rule for better maintainability.
//!  Less common commands are grouped in the 'other' subrule.

pub mod awk;
pub mod bat;
pub mod cat;
pub mod cd;
pub mod date;
pub mod df;
pub mod echo;
pub mod find;
pub mod free;
pub mod grep;
pub mod head;
pub mod hostname;
pub mod ls;
pub mod pagers;

pub mod read;
pub mod sed;
pub mod sort;
pub mod tac;
pub mod tail;
pub mod uname;
pub mod uniq;
pub mod uptime;
pub mod users;
pub mod w;
pub mod wc;
pub mod who;
