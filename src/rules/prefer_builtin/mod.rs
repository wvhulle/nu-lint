// Individual rules for prefer_builtin_* commands
//
// Each common command has its own rule for better maintainability.
// Less common commands are grouped in the 'other' subrule.

pub mod cat;
pub mod find;
pub mod grep;
pub mod head;
pub mod ls;
pub mod other;
pub mod sed;
pub mod sort;
pub mod tail;
pub mod uniq;

// Re-export rule() functions for convenience
pub use cat::rule as prefer_builtin_cat;
pub use find::rule as prefer_builtin_find;
pub use grep::rule as prefer_builtin_grep;
pub use head::rule as prefer_builtin_head;
pub use ls::rule as prefer_builtin_ls;
pub use other::rule as prefer_builtin_other;
pub use sed::rule as prefer_builtin_sed;
pub use sort::rule as prefer_builtin_sort;
pub use tail::rule as prefer_builtin_tail;
pub use uniq::rule as prefer_builtin_uniq;
