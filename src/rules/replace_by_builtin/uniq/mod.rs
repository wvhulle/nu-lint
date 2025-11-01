use std::collections::HashMap;

use crate::{
    RuleViolation,
    context::LintContext,
    external_command::{BuiltinAlternative, Fix, extract_external_args},
    rule::{Rule, RuleCategory},
    violation::{Replacement, Severity},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert(
        "uniq",
        BuiltinAlternative::with_note(
            "uniq or uniq-by",
            "Use Nu's 'uniq' for removing duplicates, 'uniq-by' for column-based deduplication. \
             Nu's uniq works on structured data and provides --count flag for counting \
             occurrences.",
        ),
    );
    map
}

/// Parse uniq command arguments to extract key options
#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
struct UniqOptions {
    count: bool,
    repeated: bool,
    unique: bool,
    ignore_case: bool,
    skip_fields: Option<usize>,
}

impl UniqOptions {
    fn parse(args: &[String]) -> Self {
        let mut opts = Self::default();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
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

fn build_fix(
    _cmd_text: &str,
    _alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text = extract_external_args(args, context);
    let opts = UniqOptions::parse(&args_text);
    let (replacement, description) = opts.to_nushell();

    Fix {
        description: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            new_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    crate::external_command::detect_external_commands(
        context,
        "prefer_builtin_uniq",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_uniq",
        RuleCategory::Idioms,
        Severity::Info,
        "Use Nu's 'uniq' command for structured data support",
        check,
    )
}

#[cfg(test)]
mod basic_conversion;
#[cfg(test)]
mod flag_parsing;
