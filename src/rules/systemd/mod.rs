//! Systemd-related lint rules and shared utilities.
//!
//! This module contains rules for ensuring proper systemd compatibility,
//! particularly for journal log level prefixes.

mod journal_prefix;

pub mod add_journal_prefix;
pub mod mnemonic_log_level;

pub use journal_prefix::{
    FixGenerator, LogLevel, PrefixStatus, extract_first_string_part, is_print_or_echo,
    pipeline_contains_print,
};
