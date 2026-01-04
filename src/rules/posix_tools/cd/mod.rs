use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
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

struct UseBuiltinCd;

impl DetectFix for UseBuiltinCd {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_cd"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's built-in 'cd' instead of external cd command"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/cd.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // External cd is always wrong - it can't change the shell's directory
        // This is a conceptual error, not a translation issue
        context.detect_external_with_validation("cd", |_, _| Some(NOTE))
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = CdOptions::parse(fix_data.arg_strings.iter().copied());
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

pub static RULE: &dyn Rule = &UseBuiltinCd;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
