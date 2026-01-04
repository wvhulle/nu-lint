use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use Nu's 'uniq' for removing duplicates, 'uniq-by' for column-based \
                    deduplication. Nu's uniq works on structured data and provides --count flag \
                    for counting occurrences.";

/// Parse uniq command arguments to extract key options
#[derive(Default)]

struct UniqOptions {
    count: bool,
    repeated: bool,
    unique: bool,
    ignore_case: bool,
    skip_fields: Option<usize>,
}

impl UniqOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            match arg {
                "-c" | "--count" => opts.count = true,
                "-d" | "--repeated" => opts.repeated = true,
                "-u" | "--unique" => opts.unique = true,
                "-i" | "--ignore-case" => opts.ignore_case = true,
                "-f" | "--skip-fields" => {
                    opts.skip_fields = iter.next().and_then(|s| s.parse().ok());
                }
                _ if arg.starts_with("-f") && arg.len() > 2 => {
                    opts.skip_fields = arg[2..].parse().ok();
                }
                _ if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 2 => {
                    Self::parse_combined_flags(&mut opts, arg);
                }
                _ => {}
            }
        }

        opts
    }

    fn parse_combined_flags(opts: &mut Self, arg: &str) {
        for ch in arg.chars().skip(1) {
            match ch {
                'c' => opts.count = true,
                'd' => opts.repeated = true,
                'u' => opts.unique = true,
                'i' => opts.ignore_case = true,
                _ => {}
            }
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let mut parts = vec!["uniq".to_string()];
        let mut examples = Vec::new();

        if self.count {
            parts.push("--count".to_string());
            examples.push("count: use --count flag (same as Unix uniq -c)".to_string());
        }

        if self.repeated {
            examples.push(
                "repeated only: use 'uniq --count | where count > 1' to show only duplicates"
                    .to_string(),
            );
        }

        if self.unique {
            examples.push(
                "unique only: use 'uniq --count | where count == 1' to show only unique items"
                    .to_string(),
            );
        }

        if self.ignore_case {
            examples.push("case-insensitive: pipe to 'str downcase' before 'uniq'".to_string());
        }

        if self.skip_fields.is_some() {
            examples.push("field-based: use 'uniq-by <column>' for structured data".to_string());
        }

        let replacement = parts.join(" ");
        let description = self.build_description(&examples);

        (replacement, description)
    }

    fn build_description(&self, examples: &[String]) -> String {
        let mut parts =
            vec!["Use Nu's built-in 'uniq' which works on structured data.".to_string()];

        if !examples.is_empty() {
            parts.push(format!("Conversions: {}", examples.join("; ")));
        }

        if self.skip_fields.is_some() {
            parts.push(
                "For field-based deduplication, use 'uniq-by <column>' which works with \
                 structured data columns."
                    .to_string(),
            );
        }

        if self.ignore_case {
            parts.push(
                "For case-insensitive uniqueness, pipe to 'str downcase' before 'uniq'."
                    .to_string(),
            );
        }

        parts.push(
            "Nu's uniq integrates with structured data, enabling operations like 'uniq-by' for \
             specific columns."
                .to_string(),
        );

        parts.join(" ")
    }
}

struct UseBuiltinUniq;

impl DetectFix for UseBuiltinUniq {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_uniq"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'uniq' command for structured data support"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/uniq.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("uniq", |_, args| {
            // Only exclude very complex uniq options
            let has_very_complex = args.iter().any(|arg| {
                matches!(
                    *arg,
                    "-z" | "--zero-terminated" |   // Null terminated
                    "--group" // Group adjacent duplicates
                )
            });
            if has_very_complex { None } else { Some(NOTE) }
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = UniqOptions::parse(fix_data.arg_strings.iter().copied());
        let (replacement, description) = opts.to_nushell();

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinUniq;

#[cfg(test)]
mod basic_conversion;
#[cfg(test)]
mod flag_parsing;
