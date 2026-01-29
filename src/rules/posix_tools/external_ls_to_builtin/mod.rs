use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use Nu's built-in 'ls' which returns structured table data (name, type, size, \
                    modified) enabling data manipulation through pipes. Unlike Unix ls, Nu's ls \
                    always provides consistent structured output without parsing.";

/// Parse ls command arguments to extract key options
#[derive(Default)]

struct LsOptions {
    paths: Vec<String>,
    all: bool,
    long: bool,
    human_readable: bool,
    recursive: bool,
    sort_by_time: bool,
    sort_by_size: bool,
    reverse: bool,
    directory: bool,
}

impl LsOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();

        for text in args {
            Self::parse_arg(&mut opts, text);
        }

        opts
    }

    fn parse_arg(opts: &mut Self, text: &str) {
        if text.starts_with('-') && !text.starts_with("--") {
            // Handle combined short flags like -la
            Self::parse_short_flags(opts, text);
        } else {
            Self::parse_long_flag(opts, text);
        }
    }

    fn parse_short_flags(opts: &mut Self, arg: &str) {
        for ch in arg.chars().skip(1) {
            match ch {
                'a' | 'A' => opts.all = true,
                'l' => opts.long = true,
                'h' => opts.human_readable = true,
                'R' => opts.recursive = true,
                't' => opts.sort_by_time = true,
                'S' => opts.sort_by_size = true,
                'r' => opts.reverse = true,
                'd' => opts.directory = true,
                _ => {}
            }
        }
    }

    fn parse_long_flag(opts: &mut Self, text: &str) {
        match text {
            "--all" => opts.all = true,
            "--human-readable" => opts.human_readable = true,
            "--recursive" => opts.recursive = true,
            "--reverse" => opts.reverse = true,
            "--directory" => opts.directory = true,
            s if !s.starts_with('-') => opts.paths.push(s.to_string()),
            _ => {}
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let mut parts = vec!["ls".to_string()];
        let mut examples = Vec::new();

        // Add path arguments
        if !self.paths.is_empty() {
            parts.extend(self.paths.iter().cloned());
        }

        // Nu ls flags that work the same
        if self.all {
            parts.push("--all".to_string());
            examples.push("--all: show hidden files (same as Unix ls -a)".to_string());
        }

        // Build pipeline transformations for flags that need conversion
        let mut pipeline = Vec::new();

        if self.sort_by_time {
            pipeline.push("sort-by modified");
            examples.push("sort by time: use 'sort-by modified' instead of -t".to_string());
        }

        if self.sort_by_size {
            pipeline.push("sort-by size");
            examples.push("sort by size: use 'sort-by size' instead of -S".to_string());
        }

        if self.reverse {
            pipeline.push("reverse");
            examples.push("reverse order: use 'reverse' instead of -r".to_string());
        }

        let replacement = if pipeline.is_empty() {
            parts.join(" ")
        } else {
            format!("{} | {}", parts.join(" "), pipeline.join(" | "))
        };

        let description = self.build_description(&examples);

        (replacement, description)
    }

    fn build_description(&self, examples: &[String]) -> String {
        let mut parts = vec![
            "Use Nu's built-in 'ls' which returns structured data (name, type, size, modified)."
                .to_string(),
        ];

        if !examples.is_empty() {
            parts.push(format!("Conversions: {}", examples.join("; ")));
        }

        if self.long || self.human_readable {
            parts.push(
                "Note: -l and -h flags are not needed in Nu. The ls command always shows detailed \
                 information in a structured table, and sizes are automatically human-readable."
                    .to_string(),
            );
        }

        if self.recursive {
            parts.push(
                "For recursive listing, use glob patterns like 'ls **/*' which is more powerful \
                 than -R."
                    .to_string(),
            );
        }

        parts.push(
            "Unlike Unix ls, Nu's ls provides consistent structured output enabling data \
             manipulation with 'where', 'sort-by', 'group-by', etc."
                .to_string(),
        );

        parts.join(" ")
    }
}

struct UseBuiltinLs;

impl DetectFix for UseBuiltinLs {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "external_ls_to_builtin"
    }

    fn short_description(&self) -> &'static str {
        "External `ls` replaceable with built-in"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/ls.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // ls/exa/eza all work well with Nu's structured ls command
        // Most common flags translate cleanly
        let mut violations = context.detect_external_with_validation("ls", |_, _, _| Some(NOTE));
        for cmd in ["exa", "eza"] {
            violations.extend(context.detect_external_with_validation(cmd, |_, _, _| Some(NOTE)));
        }
        violations
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = LsOptions::parse(fix_data.arg_texts(context));
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

pub static RULE: &dyn Rule = &UseBuiltinLs;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
