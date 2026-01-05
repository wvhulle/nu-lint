use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use Nu's 'glob' for pattern matching or 'ls' for type filtering. 'glob \
                    **/*.ext' returns file paths. 'ls **/*.ext' returns structured data (name, \
                    type, size, modified) for filtering with 'where'.";

#[derive(Default)]
struct FdOptions {
    pattern: Option<String>,
    path: Option<String>,
    file_type: Option<String>,
    extension: Option<String>,
    glob_mode: bool,
}

impl FdOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter();
        let mut positional = 0;

        while let Some(arg) = iter.next() {
            match arg {
                "-t" | "--type" => opts.file_type = iter.next().map(str::to_string),
                "-e" | "--extension" => opts.extension = iter.next().map(str::to_string),
                "-g" | "--glob" => opts.glob_mode = true,
                "-d" | "--max-depth" | "-E" | "--exclude" | "-S" | "--size"
                | "--changed-within" | "--changed-before" => {
                    iter.next();
                }
                s if s.starts_with('-') => {}
                val => {
                    match positional {
                        0 => opts.pattern = Some(val.to_string()),
                        1 => opts.path = Some(val.to_string()),
                        _ => {}
                    }
                    positional += 1;
                }
            }
        }
        opts
    }

    fn to_nushell(&self) -> (String, String) {
        let base = self.path.as_deref().unwrap_or(".");
        let pattern = self.build_glob_pattern(base);

        self.type_filter().map_or_else(
            || {
                let replacement = format!("glob {pattern}");
                let description = format!(
                    "Use 'glob {pattern}' to find files. '**' recursively matches subdirectories."
                );
                (replacement, description)
            },
            |type_filter| {
                let replacement = format!("ls {pattern} | {type_filter}");
                let description = format!(
                    "Use 'ls {pattern} | {type_filter}'. ls returns structured data for filtering."
                );
                (replacement, description)
            },
        )
    }

    fn build_glob_pattern(&self, base: &str) -> String {
        if let Some(ext) = self.extension.as_deref() {
            return format!("{base}/**/*.{ext}");
        }
        self.pattern.as_deref().map_or_else(
            || format!("{base}/**/*"),
            |p| {
                if p.contains('*') || self.glob_mode {
                    format!("{base}/**/{p}")
                } else {
                    format!("{base}/**/*{p}*")
                }
            },
        )
    }

    fn type_filter(&self) -> Option<&'static str> {
        self.file_type.as_deref().and_then(|t| match t {
            "f" | "file" => Some("where type == file"),
            "d" | "directory" => Some("where type == dir"),
            "l" | "symlink" => Some("where type == symlink"),
            _ => None,
        })
    }
}

struct UseBuiltinFd;

impl DetectFix for UseBuiltinFd {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_fd"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'glob' or 'ls' instead of 'fd'"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/glob.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("fd", |_, _, _| Some(NOTE))
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = FdOptions::parse(fix_data.arg_texts(context));
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
