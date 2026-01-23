//! DSL-to-Nushell conversion utilities
//!
//! This module provides conversion logic for various domain-specific languages
//! to equivalent Nushell commands.

use std::borrow::Cow;

pub mod jq;

/// Context for DSL-to-Nushell conversion.
/// Shared between different DSL translators.
#[derive(Clone, Default)]
pub enum ConversionContext {
    /// Data comes from stdin/pipeline - no file open needed
    #[default]
    Pipeline,
    /// Data comes from a file - prepend `open <file> | from json |`
    File(String),
}

impl ConversionContext {
    /// Wrap a Nu command string with the appropriate prefix based on context
    pub fn wrap_str(&self, cmd: &str) -> Cow<'static, str> {
        match self {
            Self::Pipeline => Cow::Owned(cmd.to_string()),
            Self::File(file_text) => Cow::Owned(format!("open {file_text} | from json | {cmd}")),
        }
    }
}
