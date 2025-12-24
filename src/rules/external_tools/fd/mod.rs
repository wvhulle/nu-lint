use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'ls **/*.ext' for recursive file matching or 'glob **/*.ext' for pattern \
                    matching. While fd is a modern alternative to bash find with better \
                    performance and UX, Nushell's ls provides structured table data that \
                    integrates seamlessly with Nushell's data manipulation commands. This enables \
                    operations like sorting, filtering, and transforming file lists without \
                    parsing text output.";

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
            format!("ls {glob_pattern}")
        } else {
            format!("ls {glob_pattern} | {}", filters.join(" | "))
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
            "Use 'ls {glob_pattern}' for recursive file search."
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
                "Note: Nushell's ls does not show hidden files by default; use 'ls -a' to include \
                 them."
                    .to_string(),
            );
        }

        parts.push(
            "Nushell's ls returns structured data (name, type, size, modified) enabling data \
             manipulation with 'where', 'sort-by', 'group-by', etc., without text parsing."
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
    let opts = FdOptions::parse(external_args_slices(args, context));
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
    detect_external_commands(context, "fd", NOTE, Some(build_fix))
}

pub const RULE: Rule = Rule::new(
    "use_builtin_fd",
    "Use Nu's 'ls' with glob patterns instead of 'fd' command",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/commands/docs/glob.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
