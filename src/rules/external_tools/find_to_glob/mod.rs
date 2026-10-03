use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel,
    ast::{external::literal_content, string::quote_nu_string},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const GLOB_SPECIAL_CHARS: &[char] = &['*', '?', '[', ']', '{', '}'];
const ALTERNATION_CHARS: &[char] = &['{', '}', '/'];

const NOTE: &str = "'glob' walks the directory tree without an external process and returns a \
                    list of absolute paths, including hidden entries, like 'find'.";

enum EntryType {
    Any,
    File,
    Directory,
}

struct FileSearch<'a> {
    directory: &'a str,
    name: Option<&'a str>,
    entry_type: EntryType,
}

impl<'a> FileSearch<'a> {
    fn parse(args: &'a [ExternalArgument]) -> Option<Self> {
        let mut words = args
            .iter()
            .map(|argument| match argument {
                ExternalArgument::Regular(expr) => literal_content(expr),
                ExternalArgument::Spread(_) => None,
            })
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .peekable();

        let directory = match words.peek() {
            Some(first) if !first.starts_with('-') => words.next()?,
            _ => ".",
        };
        if directory.contains(GLOB_SPECIAL_CHARS) {
            return None;
        }

        let mut search = Self {
            directory: directory.trim_end_matches('/'),
            name: None,
            entry_type: EntryType::Any,
        };
        while let Some(predicate) = words.next() {
            match (predicate, words.next()?) {
                ("-name", name)
                    if search.name.is_none()
                        && !name.contains(ALTERNATION_CHARS)
                        && !name.contains("**") =>
                {
                    search.name = Some(name);
                }
                ("-type", "f") => search.entry_type = EntryType::File,
                ("-type", "d") => search.entry_type = EntryType::Directory,
                _ => return None,
            }
        }
        Some(search)
    }

    fn to_glob(&self) -> String {
        let directory = self.directory;
        let pattern = self.name.map_or_else(
            || format!("{directory}/**"),
            |name| format!("{directory}/**/{name}"),
        );
        let filter = match self.entry_type {
            EntryType::Any => "",
            EntryType::File => " --no-dir --no-symlink",
            EntryType::Directory => " --no-file --no-symlink",
        };
        format!("glob {}{filter}", quote_nu_string(&pattern))
    }
}

struct FindToGlob;

impl DetectFix for FindToGlob {
    type FixInput<'a> = Replacement;

    fn id(&self) -> &'static str {
        "find_to_glob"
    }

    fn short_description(&self) -> &'static str {
        "`find` by name or type replaceable with `glob`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/glob.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["find"])
            .iter()
            .filter_map(|invocation| {
                let search = FileSearch::parse(invocation.args)?;
                let replacement = Replacement::new(invocation.span, search.to_glob());
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'find' matching names");
                Some((detection, replacement))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Search with '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
        })
    }
}

pub static RULE: &dyn Rule = &FindToGlob;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
