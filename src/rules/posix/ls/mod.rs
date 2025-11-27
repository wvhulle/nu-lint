use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
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

        for arg in args {
            Self::parse_arg(&mut opts, arg);
        }

        opts
    }

    fn parse_arg(opts: &mut Self, arg: &str) {
        if arg.starts_with('-') && !arg.starts_with("--") {
            // Handle combined short flags like -la
            Self::parse_short_flags(opts, arg);
        } else {
            Self::parse_long_flag(opts, arg);
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

    fn parse_long_flag(opts: &mut Self, arg: &str) {
        match arg {
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

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let opts = LsOptions::parse(external_args_slices(args, context));
    let (replacement, description) = opts.to_nushell();

    Fix {
        explanation: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            replacement_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    // External ls
    violations.extend(detect_external_commands(
        context,
        "prefer_builtin_ls",
        "ls",
        NOTE,
        Some(build_fix),
    ));

    // exa/eza alternatives commonly used
    for cmd in ["exa", "eza"] {
        violations.extend(detect_external_commands(
            context,
            "prefer_builtin_ls",
            cmd,
            NOTE,
            Some(build_fix),
        ));
    }

    violations
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_ls",
        "Use Nu's built-in 'ls' instead of external ls command for structured data",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/ls.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
