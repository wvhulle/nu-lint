use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'ls **/*.ext' for recursive file matching or 'glob **/*.ext' for pattern \
                    matching. Nushell's ls returns structured table data (name, type, size, \
                    modified) instead of plain text, enabling powerful data manipulation through \
                    pipes. Note: Nushell's built-in 'find' command (without ^) is for \
                    searching/filtering data in structures, not for finding files.";

/// Parse find command arguments to extract key options
#[derive(Default)]
struct FindOptions<'a> {
    path: Option<&'a str>,
    name_pattern: Option<&'a str>,
    file_type: Option<&'a str>,
    size: Option<&'a str>,
    mtime: Option<&'a str>,
    empty: bool,
}

impl<'a> FindOptions<'a> {
    fn parse(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            match arg {
                "-name" | "-iname" => {
                    opts.name_pattern = iter.next();
                }
                "-type" => {
                    opts.file_type = iter.next();
                }
                "-size" => {
                    opts.size = iter.next();
                }
                "-mtime" | "-mmin" => {
                    opts.mtime = iter.next();
                }
                "-empty" => {
                    opts.empty = true;
                }
                // Skip unsupported options with values
                "-maxdepth" | "-mindepth" | "-newer" | "-executable" | "-perm" => {
                    iter.next();
                }
                s if !s.starts_with('-') && opts.path.is_none() => {
                    opts.path = Some(s);
                }
                _ => {}
            }
        }

        opts
    }

    fn to_nushell(&self) -> (String, String) {
        let base_path = self.path.unwrap_or(".");

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
        self.name_pattern.map_or_else(
            || format!("{base_path}/**/*"),
            |pattern| {
                let clean = pattern.trim_matches('"').trim_matches('\'');
                if clean.contains('*') {
                    format!("{base_path}/**/{clean}")
                } else {
                    format!("{base_path}/**/*{clean}*")
                }
            },
        )
    }

    fn build_filters(&self) -> (Vec<String>, Vec<String>) {
        let mut filters = Vec::new();
        let mut examples = Vec::new();

        // Type filter
        if let Some((filter, example)) = self.file_type.and_then(|ftype| match ftype {
            "f" => Some(("where type == file", "type: 'where type == file'")),
            "d" => Some(("where type == dir", "type: 'where type == dir'")),
            "l" => Some(("where type == symlink", "type: 'where type == symlink'")),
            _ => None,
        }) {
            filters.push(filter.to_string());
            examples.push(example.to_string());
        }

        // Size filter
        if let Some(size) = self.size {
            let filter = parse_size_filter(size);
            examples.push(format!("size: '{filter}'"));
            filters.push(filter);
        }

        // Time filter
        if let Some(mtime) = self.mtime {
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
    let (op, value) = size.strip_prefix('+').map_or_else(
        || {
            size.strip_prefix('-')
                .map_or(("==", size), |stripped| ("<", stripped))
        },
        |stripped| (">", stripped),
    );

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
    let (op, days) = mtime.strip_prefix('+').map_or_else(
        || {
            mtime
                .strip_prefix('-')
                .map_or((">", mtime), |stripped| (">", stripped))
        },
        |stripped| ("<", stripped),
    );

    format!("where modified {op} ((date now) - {days}day)")
}

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let opts = FindOptions::parse(external_args_slices(args, context));
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
    detect_external_commands(context, "find", NOTE, Some(build_fix))
}

pub const RULE: Rule = Rule::new(
    "use_builtin_find",
    "Use Nu's 'ls' with glob patterns instead of 'find' command",
    check,
    LintLevel::Warning,
)
.with_doc_url("https://www.nushell.sh/commands/docs/glob.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
