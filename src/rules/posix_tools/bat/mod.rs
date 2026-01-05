use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'open' to read files as structured data, or 'open --raw' for plain text. \
                    While bat provides syntax highlighting, Nu's open auto-detects file formats \
                    (JSON, TOML, CSV, etc.) and parses them into structured tables.";

const STRUCTURED_EXTENSIONS: &[&str] = &[
    ".json", ".toml", ".yaml", ".yml", ".csv", ".tsv", ".xml", ".nuon", ".ini", ".ics", ".eml",
    ".vcf", ".xlsx", ".xls", ".ods", ".db", ".sqlite",
];

fn is_structured_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    STRUCTURED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

struct UseBuiltinBat;

impl DetectFix for UseBuiltinBat {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_bat"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'open' command instead of 'bat' for file viewing"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/open.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // bat/batcat are essentially cat with syntax highlighting
        // Nu's open provides similar functionality for viewing files
        let mut violations = context.detect_external_with_validation("bat", |_, _| Some(NOTE));
        violations.extend(context.detect_external_with_validation("batcat", |_, _| Some(NOTE)));
        violations
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let has_complex_flags = fix_data.arg_strings(_context).any(|s| {
            matches!(
                s,
                "--language" | "-l" | "--theme" | "--style" | "--paging" | "--color"
            ) || s.starts_with("--language=")
                || s.starts_with("--theme=")
                || s.starts_with("--style=")
        });

        if has_complex_flags {
            return None;
        }

        let filename = fix_data.arg_strings(_context).find(|s| !s.starts_with('-'));

        let (replacement, description) = filename.map_or_else(
            || {
                (
                    "open --raw".to_string(),
                    "Use 'open --raw' for plain text files. For structured files (JSON, TOML, \
                     CSV), use 'open' without --raw to get parsed data."
                        .to_string(),
                )
            },
            |file| {
                if is_structured_file(file) {
                    (
                        format!("open {file}"),
                        format!(
                            "Use 'open {file}' to auto-parse this structured file. Nu will detect \
                             the format and return a table/record you can query directly."
                        ),
                    )
                } else {
                    (
                        format!("open --raw {file}"),
                        "Use 'open --raw' for plain text files. For structured files (JSON, TOML, \
                         CSV), use 'open' without --raw to get parsed data."
                            .to_string(),
                    )
                }
            },
        );

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinBat;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
