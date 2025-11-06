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
        "cat",
        BuiltinAlternative::with_note(
            "open --raw",
            "Use 'open' to read files as structured data, or 'open --raw' for plain text. Nu's \
             open auto-detects file formats (JSON, TOML, CSV, etc.) and parses them into \
             structured tables.",
        ),
    );
    map.insert(
        "tac",
        BuiltinAlternative::with_note(
            "open --raw | lines | reverse",
            "Use 'open --raw | lines | reverse' to reverse file lines. Nu's structured approach \
             is more explicit than tac and works with any data source.",
        ),
    );
    map.insert(
        "more",
        BuiltinAlternative::with_note(
            "open --raw",
            "Use 'open --raw' to read files. Nu displays data in tables automatically. For \
             interactive paging, pipe to 'table' which provides scrolling for large datasets.",
        ),
    );
    map.insert(
        "less",
        BuiltinAlternative::with_note(
            "open --raw",
            "Use 'open --raw' to read files. Nu's table view provides automatic paging for large \
             data. For plain text, 'open --raw' gives you the content to pipe through Nu's data \
             commands.",
        ),
    );
    map
}

/// Parse cat command arguments to extract key options
#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
struct CatOptions {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank: bool,
    show_ends: bool,
    show_tabs: bool,
    show_all: bool,
}

impl CatOptions {
    fn parse(args: &[String]) -> Self {
        let mut opts = Self::default();

        for arg in args {
            Self::parse_arg(&mut opts, arg);
        }

        opts
    }

    fn parse_arg(opts: &mut Self, arg: &str) {
        match arg {
            "-n" | "--number" => opts.number_lines = true,
            "-b" | "--number-nonblank" => opts.number_nonblank = true,
            "-E" | "--show-ends" => opts.show_ends = true,
            "-T" | "--show-tabs" => opts.show_tabs = true,
            "-A" | "--show-all" => opts.show_all = true,
            s if !s.starts_with('-') => opts.files.push(s.to_string()),
            _ => {}
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let file_args = if self.files.is_empty() {
            String::new()
        } else if self.files.len() == 1 {
            self.files[0].clone()
        } else {
            self.files.join(" ")
        };

        // Check if we need any post-processing
        let needs_processing = self.number_lines
            || self.number_nonblank
            || self.show_ends
            || self.show_tabs
            || self.show_all;

        let (replacement, description) = if needs_processing {
            self.build_with_processing(&file_args)
        } else if self.files.len() > 1 {
            self.build_multiple_files()
        } else {
            Self::build_simple(&file_args)
        };

        (replacement, description)
    }

    fn build_simple(file_arg: &str) -> (String, String) {
        let replacement = if file_arg.is_empty() {
            "open --raw".to_string()
        } else {
            format!("open --raw {file_arg}")
        };

        let description = "Use 'open --raw' for plain text, or just 'open' to auto-parse \
                           structured files (JSON, TOML, CSV, etc.). Nu's open returns data you \
                           can immediately manipulate in pipelines."
            .to_string();

        (replacement, description)
    }

    fn build_multiple_files(&self) -> (String, String) {
        let file_list = self.files.join(" ");
        let replacement = format!("[{file_list}] | each {{|f| open --raw $f}} | str join");

        let description = format!(
            "Use 'each' with 'open --raw' to read multiple files ({}). The results are joined \
             into a single string. This provides more control than cat.",
            self.files.len()
        );

        (replacement, description)
    }

    fn build_with_processing(&self, file_arg: &str) -> (String, String) {
        let mut pipeline = vec![];
        let mut examples = vec![];

        let base = if file_arg.is_empty() {
            "open --raw".to_string()
        } else {
            format!("open --raw {file_arg}")
        };

        pipeline.push(base);

        // Convert to lines for processing
        pipeline.push("lines".to_string());

        if self.number_lines || self.number_nonblank {
            pipeline.push("enumerate".to_string());
            if self.number_nonblank {
                examples.push(
                    "-b (number non-blank): use 'enumerate' after filtering empty lines"
                        .to_string(),
                );
                pipeline.push("where $it.item != \"\"".to_string());
            } else {
                examples.push("-n (number lines): use 'enumerate' to add line numbers".to_string());
            }
        }

        if self.show_ends || self.show_all {
            examples.push("-E (show ends): line endings are visible in Nu strings".to_string());
        }

        if self.show_tabs || self.show_all {
            examples.push("-T (show tabs): tabs are visible in Nu strings".to_string());
        }

        let description = if examples.is_empty() {
            "Use 'open --raw | lines' to process file content line by line.".to_string()
        } else {
            format!(
                "Use 'open --raw | lines' pipeline. Conversions: {}. Nu provides structured line \
                 data instead of special characters.",
                examples.join("; ")
            )
        };

        (pipeline.join(" | "), description)
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
    let opts = CatOptions::parse(&args_text);
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
        "prefer_builtin_cat",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_cat",
        RuleCategory::Idioms,
        Severity::Warning,
        "Use Nu's 'open' command instead of 'cat' for better file handling",
        check,
    )
}

#[cfg(test)]
mod basic_conversion;
#[cfg(test)]
mod flag_parsing;
