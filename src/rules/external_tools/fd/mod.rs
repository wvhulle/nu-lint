use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'glob' for pattern-based file finding. While fd is a modern alternative \
                    to bash find with better performance and UX, Nushell's glob command provides \
                    the same functionality with structured output that integrates seamlessly with \
                    Nushell's data manipulation commands. This enables operations like sorting, \
                    filtering, and transforming file lists without parsing text output.";

#[derive(Default)]
struct FdOptions<'a> {
    pattern: Option<&'a str>,
    path: Option<&'a str>,
    file_type: Option<&'a str>,
    extension: Option<&'a str>,
    hidden: bool,
    glob_mode: bool,
}

impl<'a> FdOptions<'a> {
    fn parse(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter();
        let mut positional_index = 0;

        while let Some(arg) = iter.next() {
            match arg {
                "-t" | "--type" => {
                    opts.file_type = iter.next();
                }
                "-e" | "--extension" => {
                    opts.extension = iter.next();
                }
                "-H" | "--hidden" => {
                    opts.hidden = true;
                }
                "-g" | "--glob" => {
                    opts.glob_mode = true;
                }
                "-I" | "--no-ignore" | "-u" | "--unrestricted" | "-s" | "--case-sensitive"
                | "-i" | "--ignore-case" => {}
                s if s.starts_with('-') => {
                    Self::skip_flag_with_value(&mut iter, s);
                }
                other => {
                    Self::set_positional(&mut opts, other, &mut positional_index);
                }
            }
        }

        opts
    }

    fn skip_flag_with_value<'b>(iter: &mut impl Iterator<Item = &'b str>, flag: &str) {
        if matches!(
            flag,
            "-d" | "--max-depth"
                | "-E"
                | "--exclude"
                | "-S"
                | "--size"
                | "--changed-within"
                | "--changed-before"
        ) {
            iter.next();
        }
    }

    #[allow(
        clippy::missing_const_for_fn,
        reason = "Const fn cannot have mutable references to generic types"
    )]
    fn set_positional(opts: &mut Self, value: &'a str, positional_index: &mut usize) {
        match *positional_index {
            0 => opts.pattern = Some(value),
            1 => opts.path = Some(value),
            _ => {}
        }
        *positional_index += 1;
    }

    fn to_nushell(&self) -> (String, String) {
        let base_path = self.path.unwrap_or(".");

        let glob_pattern = self.build_glob_pattern(base_path);
        let (filters, examples) = self.build_filters();

        let replacement = if filters.is_empty() {
            format!("glob {glob_pattern}")
        } else {
            format!("glob {glob_pattern} | {}", filters.join(" | "))
        };

        let description = self.build_description(&glob_pattern, &examples);

        (replacement, description)
    }

    fn build_glob_pattern(&self, base_path: &str) -> String {
        if let Some(ext) = self.extension {
            return format!("{base_path}/**/*.{ext}");
        }

        self.pattern.map_or_else(
            || format!("{base_path}/**/*"),
            |pattern| {
                let clean = pattern.trim_matches('"').trim_matches('\'');
                if clean.contains('*') || self.glob_mode {
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

        if let Some((filter, example)) = self.file_type.and_then(|ftype| match ftype {
            "f" | "file" => Some(("where type == file", "type: 'where type == file'")),
            "d" | "directory" => Some(("where type == dir", "type: 'where type == dir'")),
            "l" | "symlink" => Some(("where type == symlink", "type: 'where type == symlink'")),
            _ => None,
        }) {
            filters.push(filter.to_string());
            examples.push(example.to_string());
        }

        (filters, examples)
    }

    fn build_description(&self, glob_pattern: &str, examples: &[String]) -> String {
        let mut parts = vec![format!(
            "Use 'glob {glob_pattern}' for pattern-based file finding."
        )];

        if self.pattern.is_some() || self.extension.is_some() {
            parts.push(
                "The '**' glob recursively matches all subdirectories, and the pattern filters \
                 file names."
                    .to_string(),
            );
        }

        if !examples.is_empty() {
            parts.push(format!(
                "Pipeline filters replace fd flags: {}.",
                examples.join(", ")
            ));
        }

        if self.hidden {
            parts.push(
                "Note: Nushell's glob shows hidden files by default (unlike fd which hides them)."
                    .to_string(),
            );
        }

        parts.push(
            "Nushell's glob returns structured data (enabling 'where', 'sort-by', 'group-by', \
             etc.) while fd returns text that requires parsing."
                .to_string(),
        );

        parts.join(" ")
    }
}

struct UseBuiltinFd;

impl DetectFix for UseBuiltinFd {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "replace_fd_with_glob"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'glob' command instead of 'fd'"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/glob.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // fd is a modern find alternative with good defaults
        // Most fd usage translates reasonably to glob
        context.detect_external_with_validation("fd", |_, _| Some(NOTE))
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = FdOptions::parse(fix_data.arg_strings.iter().copied());
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

pub static RULE: &dyn Rule = &UseBuiltinFd;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
