use std::iter::Peekable;

use crate::{
    LintLevel,
    alternatives::{ExternalCmdFixData, detect_external_commands, external_args_slices},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use Nu's built-in 'ls' which returns structured data. Nushell's ls provides \
                    structured table output that integrates seamlessly with data manipulation \
                    commands like 'where', 'sort-by', and 'select', without requiring text \
                    parsing.";

#[derive(Default)]
struct EzaOptions {
    paths: Vec<String>,
    all_files: bool,
    long_view: bool,
    recurse: bool,
    tree: bool,
    reverse: bool,
    sort_field: Option<String>,
    only_dirs: bool,
    only_files: bool,
}

impl EzaOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter().peekable();

        while let Some(arg) = iter.next() {
            if arg.starts_with("--") {
                Self::parse_long_flag(&mut opts, arg, &mut iter);
            } else if arg.starts_with('-') && arg.len() > 1 {
                Self::parse_short_flags(&mut opts, arg, &mut iter);
            } else if !arg.starts_with('-') {
                opts.paths.push(arg.to_string());
            }
        }

        opts
    }

    fn parse_long_flag<'a>(
        opts: &mut Self,
        arg: &str,
        iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) {
        match arg {
            "--all" | "--almost-all" => opts.all_files = true,
            "--long" => opts.long_view = true,
            "--recurse" => opts.recurse = true,
            "--tree" => opts.tree = true,
            "--reverse" => opts.reverse = true,
            "--only-dirs" => opts.only_dirs = true,
            "--only-files" => opts.only_files = true,
            "--level" | "--header" => {
                iter.next();
            }
            "--sort" => {
                opts.sort_field = iter.next().map(String::from);
            }
            _ if arg.starts_with("--sort=") => {
                opts.sort_field = Some(arg[7..].to_string());
            }
            _ => {}
        }
    }

    fn parse_short_flags<'a>(
        opts: &mut Self,
        arg: &str,
        iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) {
        let chars: Vec<char> = arg[1..].chars().collect();
        for (i, ch) in chars.iter().enumerate() {
            match ch {
                'a' | 'A' => opts.all_files = true,
                'l' => opts.long_view = true,
                'R' => opts.recurse = true,
                'T' => opts.tree = true,
                'r' => opts.reverse = true,
                'D' => opts.only_dirs = true,
                'f' => opts.only_files = true,
                'h' | 'L' => Self::handle_flag_with_optional_value(i, &chars, iter, &mut None),
                's' => Self::handle_flag_with_optional_value(i, &chars, iter, &mut opts.sort_field),
                _ => {}
            }
        }
    }

    fn handle_flag_with_optional_value<'a>(
        i: usize,
        chars: &[char],
        iter: &mut Peekable<impl Iterator<Item = &'a str>>,
        target: &mut Option<String>,
    ) {
        if i + 1 < chars.len() {
            *target = Some(chars[i + 1..].iter().collect());
        } else {
            *target = iter.next().map(String::from);
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let mut parts = vec!["ls".to_string()];
        let mut filters = Vec::new();

        if self.all_files {
            parts.push("-a".to_string());
        }

        if self.long_view {
            parts.push("-l".to_string());
        }

        self.add_path_patterns(&mut parts);

        if self.only_dirs {
            filters.push("where type == dir".to_string());
        } else if self.only_files {
            filters.push("where type == file".to_string());
        }

        self.add_sort_filters(&mut filters);

        let replacement = if filters.is_empty() {
            parts.join(" ")
        } else {
            format!("{} | {}", parts.join(" "), filters.join(" | "))
        };

        let description = self.build_description();

        (replacement, description)
    }

    fn add_path_patterns(&self, parts: &mut Vec<String>) {
        let suffix = if self.recurse || self.tree {
            "/**/*"
        } else {
            ""
        };

        if self.paths.is_empty() {
            if !suffix.is_empty() {
                parts.push("**/*".to_string());
            }
        } else {
            for path in &self.paths {
                parts.push(format!("{path}{suffix}"));
            }
        }
    }

    fn add_sort_filters(&self, filters: &mut Vec<String>) {
        if let Some(sort) = &self.sort_field {
            let nu_sort = Self::convert_sort_field(sort);
            if self.reverse {
                filters.push(format!("sort-by {nu_sort} --reverse"));
            } else {
                filters.push(format!("sort-by {nu_sort}"));
            }
        } else if self.reverse {
            filters.push("reverse".to_string());
        }
    }

    #[allow(
        clippy::match_same_arms,
        reason = "Explicit mapping from eza to Nu sort fields"
    )]
    fn convert_sort_field(eza_sort: &str) -> &'static str {
        match eza_sort {
            "name" | "Name" | "extension" | "Extension" => "name",
            "size" => "size",
            "modified" | "date" | "time" | "newest" => "modified",
            "accessed" => "accessed",
            "created" => "created",
            "type" => "type",
            _ => "name",
        }
    }

    fn build_description(&self) -> String {
        let mut parts = vec!["Replace eza with Nushell's ls command.".to_string()];

        if self.all_files {
            parts.push("Hidden files: -a/--all → ls -a".to_string());
        }

        if self.long_view {
            parts.push("Long view: -l/--long → ls -l (structured table output)".to_string());
        }

        if self.recurse || self.tree {
            parts.push(
                "Recursion: -R/--recurse or -T/--tree → glob pattern **/* for recursive listing"
                    .to_string(),
            );
        }

        if self.only_dirs {
            parts.push("Directories only: -D/--only-dirs → where type == dir".to_string());
        } else if self.only_files {
            parts.push("Files only: -f/--only-files → where type == file".to_string());
        }

        if let Some(sort) = &self.sort_field {
            let nu_sort = Self::convert_sort_field(sort);
            parts.push(format!(
                "Sorting: --sort={sort} → sort-by {nu_sort}{}",
                if self.reverse { " --reverse" } else { "" }
            ));
        } else if self.reverse {
            parts.push("Reverse: -r/--reverse → reverse".to_string());
        }

        parts.push(
            "Benefits: structured data output enables filtering with 'where', sorting with \
             'sort-by', and transformation without text parsing."
                .to_string(),
        );

        parts.join(" ")
    }
}

struct UseBuiltinEza;

impl DetectFix for UseBuiltinEza {
    type FixInput = ExternalCmdFixData;

    fn id(&self) -> &'static str {
        "use_builtin_eza"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's built-in 'ls' instead of eza"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/ls.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        detect_external_commands(context, "eza", NOTE)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let opts = EzaOptions::parse(external_args_slices(&fix_data.args, context));
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

pub static RULE: &dyn Rule = &UseBuiltinEza;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
