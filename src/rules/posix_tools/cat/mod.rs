use crate::{
    LintLevel,
    alternatives::{ExternalCmdFixData, detect_external_commands, external_args_slices},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'open' to read files as structured data, or 'open --raw' for plain text. \
                    Nu's open auto-detects file formats (JSON, TOML, CSV, etc.) and parses them \
                    into structured tables.";

#[derive(Default)]
struct CatOptions {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank: bool,
    show_ends: bool,
    show_tabs: bool,
    show_all: bool,
}

impl CatOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
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

struct UseBuiltinCat;

impl DetectFix for UseBuiltinCat {
    type FixInput = ExternalCmdFixData;

    fn id(&self) -> &'static str {
        "use_builtin_cat"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'open' command instead of 'cat' for better file handling"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/open.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        let mut violations = detect_external_commands(context, "cat", NOTE);
        // Related commands commonly used like cat
        for cmd in ["tac", "more", "less"] {
            violations.extend(detect_external_commands(context, cmd, NOTE));
        }
        violations
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let opts = CatOptions::parse(external_args_slices(&fix_data.args, context));
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

pub static RULE: &dyn Rule = &UseBuiltinCat;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
