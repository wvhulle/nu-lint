use std::collections::HashMap;

use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    ast::ext_command::{BuiltinAlternative, ExternalArgumentExt, detect_external_commands},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert(
        "grep",
        BuiltinAlternative::with_note(
            "find or where",
            "Use 'find' for simple text search (case-insensitive by default), 'where $it =~ \
             pattern' for regex filtering, or 'lines | where' for line-based filtering with \
             structured data operations.",
        ),
    );
    map.insert(
        "rg",
        BuiltinAlternative::with_note(
            "find or where",
            "Use 'find' for simple text search or 'where $it =~ pattern' for regex filtering. \
             While ripgrep is fast, Nushell's find and where work on structured data and \
             integrate seamlessly with pipelines.",
        ),
    );
    map
}

/// Grep option flags
#[derive(Default)]

struct GrepFlags {
    case_insensitive: bool,
    invert_match: bool,
    line_number: bool,
    count: bool,
    files_with_matches: bool,
}

/// Parse grep command arguments to extract key options
#[derive(Default)]
struct GrepOptions {
    pattern: Option<String>,
    files: Vec<String>,
    flags: GrepFlags,
    extended_regex: bool,
    fixed_strings: bool,
    recursive: bool,
}

impl GrepOptions {
    fn parse(args: &[String]) -> Self {
        let mut opts = Self::default();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-i" | "--ignore-case" => opts.flags.case_insensitive = true,
                "-v" | "--invert-match" => opts.flags.invert_match = true,
                "-n" | "--line-number" => opts.flags.line_number = true,
                "-c" | "--count" => opts.flags.count = true,
                "-l" | "--files-with-matches" => opts.flags.files_with_matches = true,
                "-E" | "--extended-regexp" => opts.extended_regex = true,
                "-F" | "--fixed-strings" => opts.fixed_strings = true,
                "-r" | "-R" | "--recursive" => opts.recursive = true,
                // Skip unsupported options with values
                "-A" | "--after-context" | "-B" | "--before-context" | "-C" | "--context"
                | "-m" | "--max-count" => {
                    iter.next();
                }
                s if !s.starts_with('-') => Self::add_non_flag_arg(&mut opts, s),
                _ => {}
            }
        }

        opts
    }

    fn add_non_flag_arg(opts: &mut Self, arg: &str) {
        if opts.pattern.is_none() {
            opts.pattern = Some(arg.to_string());
        } else {
            opts.files.push(arg.to_string());
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let pattern = self.pattern.as_deref().unwrap_or("pattern");
        let clean_pattern = pattern.trim_matches('"').trim_matches('\'');

        // For simple text search without special flags, use 'find'
        // But if recursive or has files, use where with lines
        if !self.flags.invert_match
            && !self.flags.line_number
            && !self.flags.count
            && !self.flags.files_with_matches
            && !self.extended_regex
            && !self.fixed_strings
            && !self.recursive
            && self.files.is_empty()
        {
            let replacement = format!("find \"{clean_pattern}\"");
            let description = self.build_find_description(clean_pattern);
            return (replacement, description);
        }

        // For complex filtering, use 'where' with appropriate filters
        let (filter_expr, examples) = self.build_where_filter(clean_pattern);

        let replacement = if self.files.is_empty() {
            format!("lines | {filter_expr}")
        } else {
            // When files are involved, suggest using open with pipes
            format!("open {} | lines | {filter_expr}", self.files.join(" "))
        };

        let description = self.build_where_description(clean_pattern, &examples);

        (replacement, description)
    }

    fn build_find_description(&self, pattern: &str) -> String {
        let mut parts = vec![format!(
            "Use 'find \"{}\"' for simple text search.",
            pattern
        )];

        if self.flags.case_insensitive {
            parts.push(
                "The -i flag is redundant; Nu's 'find' is case-insensitive by default.".to_string(),
            );
        } else {
            parts.push("Nu's 'find' is case-insensitive by default.".to_string());
        }

        parts.push(
            "The 'find' command works on structured data, lists, and strings, making it more \
             versatile than grep."
                .to_string(),
        );

        parts.join(" ")
    }

    fn build_where_filter(&self, pattern: &str) -> (String, Vec<String>) {
        let mut examples = Vec::new();
        let mut filter_parts = Vec::new();

        // Pattern matching
        let pattern_expr = if self.fixed_strings {
            examples.push("fixed string: use 'str contains' for literal matching".to_string());
            format!("$it | str contains \"{pattern}\"")
        } else if self.flags.invert_match {
            examples.push("invert: '!~ pattern' for non-matching lines".to_string());
            format!("$it !~ \"{pattern}\"")
        } else {
            examples.push("regex: '=~ pattern' for regex matching".to_string());
            format!("$it =~ \"{pattern}\"")
        };

        filter_parts.push(format!("where {pattern_expr}"));

        // Additional transformations
        if self.flags.line_number {
            examples.push("line numbers: use 'enumerate' before filtering".to_string());
            filter_parts.insert(0, "enumerate".to_string());
        }

        if self.flags.count {
            examples.push("count: pipe to 'length' to count matches".to_string());
            filter_parts.push("length".to_string());
        }

        (filter_parts.join(" | "), examples)
    }

    fn build_where_description(&self, pattern: &str, examples: &[String]) -> String {
        let mut parts = vec![format!(
            "Use 'lines | where' to filter lines by pattern '{}'.",
            pattern
        )];

        if !examples.is_empty() {
            parts.push(format!("Pipeline stages: {}", examples.join("; ")));
        }

        if self.flags.case_insensitive {
            parts.push(
                "Note: -i flag is redundant in Nu. Use 'str downcase' before filtering for \
                 case-insensitive regex, or 'find' which is case-insensitive by default."
                    .to_string(),
            );
        }

        parts.push(
            "Nushell's 'where' operates on structured data, enabling powerful filtering without \
             text parsing."
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
    let args_text = args.extract_as_strings(context);
    let opts = GrepOptions::parse(&args_text);
    let (replacement, description) = opts.to_nushell();

    Fix {
        description: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            new_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_grep",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_grep",
        "Use Nu's 'find' or 'where' instead of 'grep' for better data handling",
        check,
    )
}

#[cfg(test)]
mod tests;
