use core::slice;
use std::collections::HashMap;

use nu_protocol::ast::ExternalArgument;

use crate::{
    RuleViolation,
    context::LintContext,
    external_command::{BuiltinAlternative, detect_external_commands, extract_external_args},
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, Severity},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert(
        "sort",
        BuiltinAlternative::with_note(
            "sort or sort-by",
            "Use Nu's 'sort' for simple sorting or 'sort-by <column>' for structured data. Nu's \
             sort works on any data type and provides natural sorting with -n flag.",
        ),
    );
    map
}

/// Parse sort command arguments to extract key options
#[derive(Default)]

struct SortOptions {
    reverse: bool,
    numeric: bool,
    unique: bool,
    key_field: Option<String>,
    ignore_case: bool,
}

impl SortOptions {
    fn parse(args: &[String]) -> Self {
        let mut opts = Self::default();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 2 {
                // Handle combined short flags like -nr
                Self::parse_combined_flags(&mut opts, arg);
            } else {
                Self::parse_single_arg(&mut opts, arg, &mut iter);
            }
        }

        opts
    }

    fn parse_combined_flags(opts: &mut Self, arg: &str) {
        for ch in arg.chars().skip(1) {
            match ch {
                'r' => opts.reverse = true,
                'n' => opts.numeric = true,
                'u' => opts.unique = true,
                'f' => opts.ignore_case = true,
                'k' if arg.len() > 2 => {
                    // Handle -k2 format embedded in combined flags
                    let rest: String = arg.chars().skip_while(|&c| c != 'k').skip(1).collect();
                    Self::set_key_field_if_not_empty(opts, rest);
                    break;
                }
                _ => {}
            }
        }
    }

    fn set_key_field_if_not_empty(opts: &mut Self, rest: String) {
        if !rest.is_empty() {
            opts.key_field = Some(rest);
        }
    }

    fn parse_single_arg(opts: &mut Self, arg: &str, iter: &mut slice::Iter<String>) {
        match arg {
            "-r" | "--reverse" => opts.reverse = true,
            "-n" | "--numeric-sort" => opts.numeric = true,
            "-u" | "--unique" => opts.unique = true,
            "-f" | "--ignore-case" => opts.ignore_case = true,
            "-k" | "--key" => {
                opts.key_field = iter.next().map(String::to_string);
            }
            "-t" | "--field-separator" => {
                // Skip the separator value for now
                iter.next();
            }
            s if s.starts_with("-k") && s.len() > 2 => {
                // Handle -k2 format
                opts.key_field = Some(s[2..].to_string());
            }
            _ => {}
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let mut parts = Vec::new();
        let mut examples = Vec::new();

        // Base command
        if let Some(field) = &self.key_field {
            parts.push(format!("sort-by {field}"));
            examples.push(format!(
                "column sorting: use 'sort-by {field}' instead of -k"
            ));
        } else {
            parts.push("sort".to_string());
        }

        // Flags
        if self.numeric {
            parts.push("--natural".to_string());
            examples.push("numeric: use --natural flag for natural number sorting".to_string());
        }

        if self.reverse {
            parts.push("--reverse".to_string());
            examples.push("reverse: use --reverse flag (same as Unix sort -r)".to_string());
        }

        let replacement = parts.join(" ");
        let description = self.build_description(&examples);

        (replacement, description)
    }

    fn build_description(&self, examples: &[String]) -> String {
        let mut parts = vec!["Use Nu's built-in 'sort' which works on any data type.".to_string()];

        if !examples.is_empty() {
            parts.push(format!("Conversions: {}", examples.join("; ")));
        }

        if self.unique {
            parts.push(
                "Note: -u flag for unique values should be handled separately with 'uniq' after \
                 sorting."
                    .to_string(),
            );
        }

        if self.ignore_case {
            parts.push(
                "For case-insensitive sorting, pipe to 'str downcase' before sorting or use \
                 'sort-by -i' flag."
                    .to_string(),
            );
        }

        parts.push(
            "Nu's sort provides structured data output enabling chaining with other commands like \
             'where', 'group-by', 'each', etc."
                .to_string(),
        );

        parts.join(" ")
    }
}

fn build_fix(
    _cmd_text: &str,
    _alternative: &BuiltinAlternative,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text = extract_external_args(args, context);
    let opts = SortOptions::parse(&args_text);
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
    detect_external_commands(
        context,
        "prefer_builtin_sort",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_sort",
        RuleCategory::Idioms,
        Severity::Warning,
        "Use Nu's 'sort' command for better data type support",
        check,
    )
}

#[cfg(test)]
mod basic_conversion;
#[cfg(test)]
mod flag_conversion;
