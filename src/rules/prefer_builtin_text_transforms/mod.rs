use std::collections::HashMap;

use crate::{
    context::LintContext,
    external_command::{BuiltinAlternative, ExternalCommandVisitor},
    lint::{Fix, Replacement, Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
    visitor::VisitContext,
};

pub struct PreferBuiltinTextTransforms;

impl PreferBuiltinTextTransforms {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Map of text transformation commands to their Nushell built-in
    /// equivalents Based on <https://www.nushell.sh/book/coming_from_bash.html#command-equivalents>
    fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
        let mut map = HashMap::new();

        // Text transformation
        map.insert(
            "sed",
            BuiltinAlternative::with_note(
                "str replace",
                "Use 'str replace' for find and replace operations",
            ),
        );
        map.insert(
            "awk",
            BuiltinAlternative::with_note(
                "where, select, or each",
                "Use 'where' for filtering, 'select' for columns, or 'each' for row-by-row \
                 processing",
            ),
        );
        map.insert(
            "cut",
            BuiltinAlternative::with_note("select", "Use 'select' to choose specific columns"),
        );
        map.insert(
            "wc",
            BuiltinAlternative::with_note(
                "length or str length",
                "Use 'length' for item count or 'str length' for character count",
            ),
        );
        map.insert(
            "tee",
            BuiltinAlternative::with_note(
                "tee",
                "Use 'tee { save file.txt }' to save while passing through",
            ),
        );
        map.insert(
            "tr",
            BuiltinAlternative::with_note(
                "str replace",
                "Use 'str replace' or 'str downcase'/'str upcase' for case conversion",
            ),
        );
        map.insert(
            "rev",
            BuiltinAlternative::with_note(
                "str reverse or reverse",
                "Use 'str reverse' for string reversal or 'reverse' for list reversal",
            ),
        );

        map
    }
}

impl Default for PreferBuiltinTextTransforms {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleMetadata for PreferBuiltinTextTransforms {
    fn id(&self) -> &'static str {
        "prefer_builtin_text_transforms"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Prefer Nushell built-in commands over external tools for text transformation (sed, awk, \
         cut, wc, tr, tee)"
    }
}

impl RegexRule for PreferBuiltinTextTransforms {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = ExternalCommandVisitor::new(
            self.id(),
            self.severity(),
            Self::get_builtin_alternatives(),
            Some(build_fix),
        );
        context.walk_ast(&mut visitor);
        visitor.into_violations()
    }
}

/// Build a Fix with appropriate replacement based on the external command
fn build_fix(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &VisitContext,
) -> Fix {
    // Extract arguments from the external call using the helper
    let args_text = context.extract_external_args(args);

    // Create command-specific replacements
    let new_text = match cmd_text {
        "sed" => {
            // ^sed 's/foo/bar/' file.txt -> open file.txt | str replace 'foo' 'bar'
            if args_text.len() >= 2 {
                // Parse sed pattern (simplified - just handle basic s/pattern/replacement/)
                let pattern = &args_text[0];
                let file = &args_text[1];
                if pattern.starts_with("'s/") || pattern.starts_with("\"s/") {
                    // Extract pattern and replacement from sed syntax
                    format!("open {file} | str replace ...")
                } else {
                    "str replace".to_string()
                }
            } else {
                alternative.command.to_string()
            }
        }
        "awk" => {
            // Complex transformation - provide general guidance
            "where | select | each".to_string()
        }
        "cut" => {
            // ^cut -d ',' -f 1 file.csv -> open file.csv | select column1
            if args_text.len() >= 2 {
                let file = args_text.last().unwrap();
                format!("open {file} | select <columns>")
            } else {
                alternative.command.to_string()
            }
        }
        "wc" => {
            // ^wc -l file.txt -> open file.txt | lines | length
            if args_text.contains(&"-l".to_string()) {
                if let Some(file) = args_text.iter().find(|a| !a.starts_with('-')) {
                    format!("open {file} | lines | length")
                } else {
                    "lines | length".to_string()
                }
            } else {
                "length".to_string()
            }
        }
        "tr" => {
            // ^tr 'a-z' 'A-Z' -> str upcase
            if args_text.len() >= 2 {
                if args_text[0].contains("a-z") && args_text[1].contains("A-Z") {
                    "str upcase".to_string()
                } else if args_text[0].contains("A-Z") && args_text[1].contains("a-z") {
                    "str downcase".to_string()
                } else {
                    "str replace".to_string()
                }
            } else {
                alternative.command.to_string()
            }
        }
        "tee" => {
            // ^tee file.txt -> tee { save file.txt }
            if let Some(file) = args_text.first() {
                format!("tee {{ save {file} }}")
            } else {
                alternative.command.to_string()
            }
        }
        "rev" => {
            // ^rev -> str reverse (for string context) or reverse (for list context)
            "str reverse".to_string()
        }
        _ => alternative.command.to_string(),
    };

    // Create the replacement
    Fix {
        description: format!("Replace '^{}' with '{}'", cmd_text, alternative.command),
        replacements: vec![Replacement {
            span: expr_span,
            new_text,
        }],
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
