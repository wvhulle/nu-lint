use crate::{
    LintLevel,
    ast::string::StringFormat,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use Nu's 'glob' for pattern matching or 'ls' for metadata filtering. 'glob \
                    **/*.ext' returns file paths. 'ls **/*.ext' returns structured data (name, \
                    type, size, modified) for filtering with 'where'. Note: Nu's 'find' (without \
                    ^) searches data structures, not filesystems.";

enum PathArg {
    Formatted(StringFormat),
    Raw(String),
}

#[derive(Default)]
struct FindOptions {
    path: Option<PathArg>,
    name_pattern: Option<String>,
    file_type: Option<String>,
    size: Option<String>,
    mtime: Option<String>,
    empty: bool,
}

impl FindOptions {
    fn parse<'a>(args: impl IntoIterator<Item = (&'a str, Option<StringFormat>)>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter().peekable();

        while let Some((arg, format)) = iter.next() {
            match arg {
                "-name" | "-iname" => {
                    opts.name_pattern = iter.next().map(|(t, _)| t.to_string());
                }
                "-type" => {
                    opts.file_type = iter.next().map(|(t, _)| t.to_string());
                }
                "-size" => {
                    opts.size = iter.next().map(|(t, _)| t.to_string());
                }
                "-mtime" | "-mmin" => {
                    opts.mtime = iter.next().map(|(t, _)| t.to_string());
                }
                "-empty" => opts.empty = true,
                "-maxdepth" | "-mindepth" | "-newer" | "-executable" | "-perm" => {
                    iter.next();
                }
                s if !s.starts_with('-') && opts.path.is_none() => {
                    opts.path = Some(
                        format.map_or_else(|| PathArg::Raw(s.to_string()), PathArg::Formatted),
                    );
                }
                _ => {}
            }
        }
        opts
    }

    fn to_nushell(&self) -> (String, String) {
        let pattern = self.build_glob_pattern();
        let filters = self.build_filters();

        if filters.is_empty() {
            let replacement = format!("glob {pattern}");
            let description = format!(
                "Use 'glob {pattern}' to find files. '**' recursively matches subdirectories. Use \
                 'ls' instead if you need to filter by type/size/time."
            );
            (replacement, description)
        } else {
            let filter_str = filters.join(" | ");
            let replacement = format!("ls {pattern} | {filter_str}");
            let description = format!(
                "Use 'ls {pattern} | {filter_str}'. ls returns structured data for filtering."
            );
            (replacement, description)
        }
    }

    fn build_glob_pattern(&self) -> String {
        let base = self.path.as_ref().map_or_else(
            || ".".to_string(),
            |p| match p {
                PathArg::Formatted(fmt) => fmt.reconstruct(fmt.content()),
                PathArg::Raw(s) => s.clone(),
            },
        );
        self.name_pattern.as_ref().map_or_else(
            || format!("{base}/**/*"),
            |p| {
                if p.contains('*') {
                    format!("{base}/**/{p}")
                } else {
                    format!("{base}/**/*{p}*")
                }
            },
        )
    }

    fn build_filters(&self) -> Vec<String> {
        let mut filters = Vec::new();

        if let Some(f) = self.file_type.as_ref().and_then(|t| match t.as_str() {
            "f" => Some("where type == file"),
            "d" => Some("where type == dir"),
            "l" => Some("where type == symlink"),
            _ => None,
        }) {
            filters.push(f.to_string());
        }

        if let Some(size) = &self.size {
            filters.push(parse_size_filter(size));
        }

        if let Some(mtime) = &self.mtime {
            filters.push(parse_time_filter(mtime));
        }

        if self.empty {
            filters.push("where size == 0b".to_string());
        }

        filters
    }
}

fn parse_size_filter(size: &str) -> String {
    let (op, value) = size.strip_prefix('+').map_or_else(
        || size.strip_prefix('-').map_or(("==", size), |s| ("<", s)),
        |s| (">", s),
    );
    format!("where size {op} {}", convert_size_to_nu(value))
}

fn convert_size_to_nu(size: &str) -> String {
    let upper = size.to_uppercase();
    for (suffix, unit) in [('K', "kb"), ('M', "mb"), ('G', "gb")] {
        if let Some(num) = upper.strip_suffix(suffix) {
            return format!("{num}{unit}");
        }
    }
    format!("{size}b")
}

fn parse_time_filter(mtime: &str) -> String {
    let (op, days) = mtime.strip_prefix('+').map_or_else(
        || mtime.strip_prefix('-').map_or((">", mtime), |s| (">", s)),
        |s| ("<", s),
    );
    format!("where modified {op} ((date now) - {days}day)")
}

struct UseBuiltinFind;

impl DetectFix for UseBuiltinFind {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "find_to_glob"
    }

    fn short_description(&self) -> &'static str {
        "Use Nu's 'glob' or 'ls' instead of external 'find'"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/glob.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("find", |_, fix_data, ctx| {
            let dominated_by_complex = fix_data.arg_texts(ctx).any(|text| {
                matches!(
                    text,
                    "-exec"
                        | "-execdir"
                        | "-ok"
                        | "-okdir"
                        | "-delete"
                        | "-fprint"
                        | "-fprint0"
                        | "-fls"
                        | "-prune"
                        | "-quit"
                        | "-samefile"
                        | "!"
                        | "-not"
                        | "("
                        | ")"
                )
            });
            if dominated_by_complex {
                None
            } else {
                Some(NOTE)
            }
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let args_with_formats = fix_data
            .arg_texts(context)
            .zip(fix_data.arg_formats(context));
        let opts = FindOptions::parse(args_with_formats);
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

pub static RULE: &dyn Rule = &UseBuiltinFind;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
