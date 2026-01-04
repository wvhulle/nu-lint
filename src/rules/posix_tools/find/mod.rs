use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use Nu's 'glob' for pattern matching or 'ls' for metadata filtering. \
                    'glob **/*.ext' returns file paths. 'ls **/*.ext' returns structured \
                    data (name, type, size, modified) for filtering with 'where'. \
                    Note: Nu's 'find' (without ^) searches data structures, not filesystems.";

#[derive(Default)]
struct FindOptions<'a> {
    path: Option<&'a str>,
    name_pattern: Option<&'a str>,
    file_type: Option<&'a str>,
    size: Option<&'a str>,
    mtime: Option<&'a str>,
    empty: bool,
}

impl<'a> FindOptions<'a> {
    fn parse(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            match arg {
                "-name" | "-iname" => opts.name_pattern = iter.next(),
                "-type" => opts.file_type = iter.next(),
                "-size" => opts.size = iter.next(),
                "-mtime" | "-mmin" => opts.mtime = iter.next(),
                "-empty" => opts.empty = true,
                "-maxdepth" | "-mindepth" | "-newer" | "-executable" | "-perm" => { iter.next(); }
                s if !s.starts_with('-') && opts.path.is_none() => opts.path = Some(s),
                _ => {}
            }
        }
        opts
    }

    fn to_nushell(&self) -> (String, String) {
        let base = self.path.unwrap_or(".");
        let pattern = self.build_glob_pattern(base);
        let filters = self.build_filters();

        if filters.is_empty() {
            let replacement = format!("glob {pattern}");
            let description = format!(
                "Use 'glob {pattern}' to find files. '**' recursively matches subdirectories. \
                 Use 'ls' instead if you need to filter by type/size/time."
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

    fn build_glob_pattern(&self, base: &str) -> String {
        self.name_pattern.map_or_else(
            || format!("{base}/**/*"),
            |p| {
                let clean = p.trim_matches('"').trim_matches('\'');
                if clean.contains('*') {
                    format!("{base}/**/{clean}")
                } else {
                    format!("{base}/**/*{clean}*")
                }
            },
        )
    }

    fn build_filters(&self) -> Vec<String> {
        let mut filters = Vec::new();

        if let Some(f) = self.file_type.and_then(|t| match t {
            "f" => Some("where type == file"),
            "d" => Some("where type == dir"),
            "l" => Some("where type == symlink"),
            _ => None,
        }) {
            filters.push(f.to_string());
        }

        if let Some(size) = self.size {
            filters.push(parse_size_filter(size));
        }

        if let Some(mtime) = self.mtime {
            filters.push(parse_time_filter(mtime));
        }

        if self.empty {
            filters.push("where size == 0b".to_string());
        }

        filters
    }
}

fn parse_size_filter(size: &str) -> String {
    let (op, value) = if let Some(s) = size.strip_prefix('+') {
        (">", s)
    } else if let Some(s) = size.strip_prefix('-') {
        ("<", s)
    } else {
        ("==", size)
    };
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
    let (op, days) = if let Some(s) = mtime.strip_prefix('+') {
        ("<", s)
    } else if let Some(s) = mtime.strip_prefix('-') {
        (">", s)
    } else {
        (">", mtime)
    };
    format!("where modified {op} ((date now) - {days}day)")
}

struct UseBuiltinFind;

impl DetectFix for UseBuiltinFind {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str { "use_builtin_find" }

    fn explanation(&self) -> &'static str { "Use Nu's 'glob' or 'ls' instead of external 'find'" }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/glob.html")
    }

    fn level(&self) -> LintLevel { LintLevel::Warning }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("find", |_, args| {
            let dominated_by_complex = args.iter().any(|arg| matches!(*arg,
                "-exec" | "-execdir" | "-ok" | "-okdir" | "-delete" |
                "-fprint" | "-fprint0" | "-fls" | "-prune" | "-quit" |
                "-samefile" | "!" | "-not" | "(" | ")"
            ));
            if dominated_by_complex { None } else { Some(NOTE) }
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = FindOptions::parse(fix_data.arg_strings.iter().copied());
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
