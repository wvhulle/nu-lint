use std::collections::HashMap;

use crate::{
    RuleViolation,
    context::LintContext,
    external_command::{BuiltinAlternative, extract_external_args},
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, Severity},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert(
        "find",
        BuiltinAlternative::with_note(
            "ls or glob",
            "Use 'ls **/*.ext' for recursive file matching or 'glob **/*.ext' for pattern \
             matching. Nushell's ls returns structured table data (name, type, size, modified) \
             instead of plain text, enabling powerful data manipulation through pipes. Note: \
             Nushell's built-in 'find' command (without ^) is for searching/filtering data in \
             structures, not for finding files.",
        ),
    );
    map.insert(
        "fd",
        BuiltinAlternative::with_note(
            "ls or glob",
            "Use 'ls **/*.ext' for recursive file matching or 'glob **/*.ext' for pattern \
             matching. While fd is a modern alternative to bash find with better performance and \
             UX, Nushell's ls provides structured table data that integrates seamlessly with \
             Nushell's data manipulation commands. This enables operations like sorting, \
             filtering, and transforming file lists without parsing text output.",
        ),
    );
    map
}

/// Parse find command arguments to extract key options
#[derive(Default)]
struct FindOptions {
    path: Option<String>,
    name_pattern: Option<String>,
    file_type: Option<String>,
    size: Option<String>,
    mtime: Option<String>,
    empty: bool,
}

impl FindOptions {
    fn parse(args: &[String]) -> Self {
        let mut opts = Self::default();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-name" | "-iname" => {
                    opts.name_pattern = iter.next().map(String::to_string);
                }
                "-type" => {
                    opts.file_type = iter.next().map(String::to_string);
                }
                "-size" => {
                    opts.size = iter.next().map(String::to_string);
                }
                "-mtime" | "-mmin" => {
                    opts.mtime = iter.next().map(String::to_string);
                }
                "-empty" => {
                    opts.empty = true;
                }
                // Skip unsupported options with values
                "-maxdepth" | "-mindepth" | "-newer" | "-executable" | "-perm" => {
                    iter.next();
                }
                s if !s.starts_with('-') && opts.path.is_none() => {
                    opts.path = Some(s.to_string());
                }
                _ => {}
            }
        }

        opts
    }

    fn to_nushell(&self) -> (String, String) {
        let base_path = self.path.as_deref().unwrap_or(".");

        let glob_pattern = self.build_glob_pattern(base_path);
        let (filters, examples) = self.build_filters();

        let replacement = if filters.is_empty() {
            format!("ls {glob_pattern}")
        } else {
            format!("ls {glob_pattern} | {}", filters.join(" | "))
        };

        let description = self.build_description(&glob_pattern, &examples);

        (replacement, description)
    }

    fn build_glob_pattern(&self, base_path: &str) -> String {
        match &self.name_pattern {
            Some(pattern) => {
                let clean = pattern.trim_matches('"').trim_matches('\'');
                if clean.contains('*') {
                    format!("{base_path}/**/{clean}")
                } else {
                    format!("{base_path}/**/*{clean}*")
                }
            }
            None => format!("{base_path}/**/*"),
        }
    }

    fn build_filters(&self) -> (Vec<String>, Vec<String>) {
        let mut filters = Vec::new();
        let mut examples = Vec::new();

        // Type filter
        if let Some((filter, example)) = self.file_type.as_deref().and_then(|ftype| match ftype {
            "f" => Some(("where type == file", "type: 'where type == file'")),
            "d" => Some(("where type == dir", "type: 'where type == dir'")),
            "l" => Some(("where type == symlink", "type: 'where type == symlink'")),
            _ => None,
        }) {
            filters.push(filter.to_string());
            examples.push(example.to_string());
        }

        // Size filter
        if let Some(size) = &self.size {
            let filter = parse_size_filter(size);
            examples.push(format!("size: '{filter}'"));
            filters.push(filter);
        }

        // Time filter
        if let Some(mtime) = &self.mtime {
            let filter = parse_time_filter(mtime);
            examples.push(format!("time: '{filter}'"));
            filters.push(filter);
        }

        // Empty filter
        if self.empty {
            filters.push("where size == 0b".to_string());
            examples.push("empty: 'where size == 0b'".to_string());
        }

        (filters, examples)
    }

    fn build_description(&self, glob_pattern: &str, examples: &[String]) -> String {
        let mut parts = vec![format!(
            "Use 'ls {glob_pattern}' for recursive file search."
        )];

        if self.name_pattern.is_some() {
            parts.push(
                "The '**' glob recursively matches all subdirectories, and the pattern filters \
                 file names."
                    .to_string(),
            );
        }

        if !examples.is_empty() {
            parts.push(format!(
                "Pipeline filters replace find flags: {}.",
                examples.join(", ")
            ));
        }

        parts.push(
            "Nushell's ls returns structured data (name, type, size, modified) enabling data \
             manipulation with 'where', 'sort-by', 'group-by', etc., without text parsing."
                .to_string(),
        );

        parts.join(" ")
    }
}

fn parse_size_filter(size: &str) -> String {
    let (op, value) = if let Some(stripped) = size.strip_prefix('+') {
        (">", stripped)
    } else if let Some(stripped) = size.strip_prefix('-') {
        ("<", stripped)
    } else {
        ("==", size)
    };

    format!("where size {op} {}", convert_size_to_nu(value))
}

fn convert_size_to_nu(size: &str) -> String {
    let size_upper = size.to_uppercase();

    for (suffix, unit) in [('K', "kb"), ('M', "mb"), ('G', "gb")] {
        if let Some(num) = size_upper.strip_suffix(suffix) {
            return format!("{num}{unit}");
        }
    }

    format!("{size}b")
}

fn parse_time_filter(mtime: &str) -> String {
    let (op, days) = if let Some(stripped) = mtime.strip_prefix('+') {
        ("<", stripped)
    } else if let Some(stripped) = mtime.strip_prefix('-') {
        (">", stripped)
    } else {
        (">", mtime)
    };

    format!("where modified {op} ((date now) - {days}day)")
}

fn build_fix(
    _cmd_text: &str,
    _alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text = extract_external_args(args, context);
    let opts = FindOptions::parse(&args_text);
    let (replacement, description) = opts.to_nushell();

    Fix {
        description: description.into(),
        replacements: vec![Replacement {
            _span: expr_span,
            new_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    crate::external_command::detect_external_commands(
        context,
        "prefer_builtin_find",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_find",
        RuleCategory::Idioms,
        Severity::Warning,
        "Use Nu's 'ls' with glob patterns instead of 'find' command",
        check,
    )
}

#[cfg(test)]
mod basic_conversion;
#[cfg(test)]
mod edge_cases;
#[cfg(test)]
mod filter_parsing;
#[cfg(test)]
mod pattern_handling;
