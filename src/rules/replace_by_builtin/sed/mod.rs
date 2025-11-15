use core::slice;
use std::collections::HashMap;

use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, RuleViolation,
    ast::ext_command::{BuiltinAlternative, ExternalArgumentExt, detect_external_commands},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert(
        "sed",
        BuiltinAlternative::with_note(
            "str replace",
            "Use 'str replace' for text substitution. Supports --all for global replacement \
             (sed's /g flag), --regex for pattern matching, and works seamlessly with structured \
             data in pipelines.",
        ),
    );
    map.insert(
        "gsed",
        BuiltinAlternative::with_note(
            "str replace",
            "Use 'str replace' instead of GNU sed for text substitution.",
        ),
    );
    map
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    detect_external_commands(
        context,
        "prefer_builtin_sed",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_sed",
        LintLevel::Warn,
        "Use Nu's 'str replace' instead of 'sed' for text substitution",
        check,
    )
}

/// Parse sed command arguments to extract the operation and parameters
#[derive(Default)]

struct SedOptions {
    pattern: Option<String>,
    in_place: bool,
    global: bool,
    extended_regex: bool,
    delete: bool,
    files: Vec<String>,
}

impl SedOptions {
    fn parse(args: &[String]) -> Self {
        let mut opts = Self::default();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-i" | "--in-place" => opts.in_place = true,
                "-E" | "-r" | "--regexp-extended" => opts.extended_regex = true,
                "-e" | "--expression" => Self::handle_expression_flag(&mut opts, &mut iter),
                s if s.starts_with('-') && !s.starts_with("--") => {
                    Self::handle_combined_flags(&mut opts, s, &mut iter);
                }
                s if !s.starts_with('-') => Self::handle_non_flag_arg(&mut opts, s),
                _ => {}
            }
        }

        opts
    }

    fn handle_expression_flag(opts: &mut Self, iter: &mut slice::Iter<String>) {
        if let Some(expr) = iter.next() {
            opts.pattern = Some(expr.clone());
        }
    }

    fn handle_combined_flags(opts: &mut Self, arg: &str, iter: &mut slice::Iter<String>) {
        for ch in arg.chars().skip(1) {
            match ch {
                'i' => opts.in_place = true,
                'E' | 'r' => opts.extended_regex = true,
                'e' => Self::handle_expression_flag(opts, iter),
                _ => {}
            }
        }
    }

    fn handle_non_flag_arg(opts: &mut Self, arg: &str) {
        if opts.pattern.is_none() {
            opts.pattern = Some(arg.to_string());
            Self::detect_operation(opts, arg);
        } else {
            opts.files.push(arg.to_string());
        }
    }

    fn detect_operation(opts: &mut Self, pattern: &str) {
        let clean = pattern.trim_matches('"').trim_matches('\'');

        // Check for global flag in substitution
        if clean.contains("/g") {
            opts.global = true;
        }

        // Check for delete operation
        if clean.starts_with('d') || clean.ends_with('d') {
            opts.delete = true;
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let pattern = self.pattern.as_deref().unwrap_or("");
        let clean_pattern = pattern.trim_matches('"').trim_matches('\'');

        // Parse sed substitution pattern s/find/replace/[g]
        if let Some(conversion) = self.parse_substitution(clean_pattern) {
            return conversion;
        }

        // Handle delete operations
        if self.delete {
            return Self::build_delete_suggestion(clean_pattern);
        }

        // Default fallback
        Self::build_default_suggestion()
    }

    fn parse_substitution(&self, pattern: &str) -> Option<(String, String)> {
        // Parse s/find/replace/[g] patterns
        if !pattern.starts_with('s') || !pattern.contains('/') {
            return None;
        }

        let parts: Vec<&str> = pattern.split('/').collect();
        if parts.len() < 3 {
            return None;
        }

        let find = parts[1];
        let replace = parts[2];
        let global_flag = parts.len() > 3 && parts[3].contains('g');
        let is_global = global_flag || self.global;

        let (replacement, description) =
            self.build_replacement_for_context(find, replace, is_global);

        Some((replacement, description))
    }

    fn build_replacement_for_context(
        &self,
        find: &str,
        replace: &str,
        global: bool,
    ) -> (String, String) {
        if self.files.is_empty() {
            Self::build_str_replace(find, replace, global)
        } else if self.in_place {
            self.build_in_place_replace(find, replace, global)
        } else {
            self.build_file_replace(find, replace, global)
        }
    }

    fn build_str_replace(find: &str, replace: &str, global: bool) -> (String, String) {
        let flag = if global { " --all" } else { "" };
        let replacement = format!("str replace{flag} '{find}' '{replace}'");

        let mut desc_parts = vec!["Use 'str replace' for text substitution.".to_string()];

        if global {
            desc_parts.push("--all flag replaces all occurrences (sed's /g flag)".to_string());
        } else {
            desc_parts.push(
                "By default replaces first occurrence (use --all for global replacement)"
                    .to_string(),
            );
        }

        desc_parts
            .push("'str replace' works on strings and structured data in pipelines.".to_string());

        (replacement, desc_parts.join(" "))
    }

    fn build_in_place_replace(&self, find: &str, replace: &str, global: bool) -> (String, String) {
        let file = self.files.first().map_or("file", String::as_str);
        let flag = if global { " --all" } else { "" };
        let replacement =
            format!("open {file} | str replace{flag} '{find}' '{replace}' | save -f {file}");

        let desc = format!(
            "For in-place file editing: open → str replace{} → save. Use --all for global \
             replacement.",
            if global { " --all" } else { "" }
        );

        (replacement, desc)
    }

    fn build_file_replace(&self, find: &str, replace: &str, global: bool) -> (String, String) {
        let file = self.files.first().map_or("file", String::as_str);
        let flag = if global { " --all" } else { "" };
        let replacement = format!("open {file} | str replace{flag} '{find}' '{replace}'");

        let desc = "Use 'open' to read file, then 'str replace' for substitution. Add '| save \
                    file' for in-place editing."
            .to_string();

        (replacement, desc)
    }

    fn build_delete_suggestion(pattern: &str) -> (String, String) {
        let desc = if pattern.contains("//d") || pattern == "d" {
            "Use 'lines | where' to filter out lines. Example: 'lines | where $it !~ \"pattern\"' \
             to delete matching lines."
        } else {
            "Use 'lines | where' to filter lines, or 'str replace' to remove patterns from text."
        };

        (
            "lines | where $it !~ 'pattern'".to_string(),
            desc.to_string(),
        )
    }

    fn build_default_suggestion() -> (String, String) {
        let replacement = "str replace 'pattern' 'replacement'".to_string();
        let description = "Use 'str replace' for text substitution. Common patterns: 'str replace \
                           \"old\" \"new\"' (first occurrence), 'str replace --all \"old\" \
                           \"new\"' (all occurrences), 'str replace --regex \"pattern\" \
                           \"replacement\"' (regex mode)."
            .to_string();

        (replacement, description)
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
    let opts = SedOptions::parse(&args_text);
    let (replacement, description) = opts.to_nushell();

    Fix {
        description: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            new_text: replacement.into(),
        }],
    }
}

#[cfg(test)]
mod tests;
