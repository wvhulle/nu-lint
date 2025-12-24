use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use Nu's built-in 'cd' command. External cd cannot change the current shell's \
                    directory - it only affects the subprocess. Nu's cd supports '-' for previous \
                    directory, '~' for home, and '--physical' (-P) for resolving symlinks.";

#[derive(Default)]
struct CdOptions {
    path: Option<String>,
    physical: bool,
}

impl CdOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();

        for arg in args {
            Self::parse_arg(&mut opts, arg);
        }

        opts
    }

    fn parse_arg(opts: &mut Self, arg: &str) {
        if arg == "-" {
            // Special case: "-" means previous directory, not a flag
            opts.path = Some(arg.to_string());
        } else if arg.starts_with('-') && !arg.starts_with("--") {
            Self::parse_short_flags(opts, arg);
        } else if arg.starts_with("--") {
            Self::parse_long_flag(opts, arg);
        } else {
            opts.path = Some(arg.to_string());
        }
    }

    fn parse_short_flags(opts: &mut Self, arg: &str) {
        for ch in arg.chars().skip(1) {
            if ch == 'P' {
                opts.physical = true;
            }
            // -L is the default behavior in both shells, so we ignore it
            // -e is bash-specific for -P error handling, ignored
        }
    }

    fn parse_long_flag(opts: &mut Self, arg: &str) {
        if arg == "--physical" {
            opts.physical = true;
        }
        // --logical is default, ignore
    }

    fn to_nushell(&self) -> (String, String) {
        let mut parts = vec!["cd".to_string()];
        let mut notes = Vec::new();

        if self.physical {
            parts.push("--physical".to_string());
            notes.push("--physical: resolve symlinks before processing '..'".to_string());
        }

        if let Some(ref path) = self.path {
            parts.push(path.clone());
        }

        let replacement = parts.join(" ");
        let description = Self::build_description(&notes);

        (replacement, description)
    }

    fn build_description(notes: &[String]) -> String {
        let mut parts = vec![
            "Use Nu's built-in 'cd' command. External cd cannot change the shell's directory."
                .to_string(),
        ];

        if !notes.is_empty() {
            parts.push(format!("Flags: {}", notes.join("; ")));
        }

        parts.push(
            "Nu's cd supports '-' for previous directory, '~' for home, and '...' for multiple \
             parent levels."
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
    let opts = CdOptions::parse(external_args_slices(args, context));
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
    detect_external_commands(context, "cd", NOTE, Some(build_fix))
}

pub const RULE: Rule = Rule::new(
    "use_builtin_cd",
    "Use Nu's built-in 'cd' instead of external cd command",
    check,
    LintLevel::Warning,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/commands/docs/cd.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
