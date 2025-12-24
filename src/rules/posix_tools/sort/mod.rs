use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use Nu's 'sort' for simple sorting or 'sort-by <column>' for structured data. \
                    Nu's sort works on any data type and provides natural sorting with -n flag.";

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
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter();

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

    fn parse_single_arg<'a, I: Iterator<Item = &'a str>>(opts: &mut Self, arg: &str, iter: &mut I) {
        match arg {
            "-r" | "--reverse" => opts.reverse = true,
            "-n" | "--numeric-sort" => opts.numeric = true,
            "-u" | "--unique" => opts.unique = true,
            "-f" | "--ignore-case" => opts.ignore_case = true,
            "-k" | "--key" => {
                opts.key_field = iter.next().map(str::to_string);
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
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let opts = SortOptions::parse(external_args_slices(args, context));
    let (replacement, description) = opts.to_nushell();

    Fix {
        explanation: description.into(),
        replacements: vec![Replacement {
            span: expr_span.into(),
            replacement_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "sort", NOTE, Some(build_fix))
}

pub const RULE: Rule = Rule::new(
    "use_builtin_sort",
    "Use Nu's 'sort' command for better data type support",
    check,
    LintLevel::Warning,
)
.with_doc_url("https://www.nushell.sh/commands/docs/sort.html");

#[cfg(test)]
mod basic_conversion;
#[cfg(test)]
mod flag_conversion;
