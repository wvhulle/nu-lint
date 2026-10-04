use nu_protocol::Span;

use crate::{
    LintLevel,
    context::LintContext,
    dsl::find::FileSearch,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "'glob' walks the directory tree without an external process and returns a \
                    list of absolute paths, including hidden entries, like 'find'.";

pub struct GlobSearch<'a> {
    span: Span,
    search: FileSearch<'a>,
}

struct FindToGlob;

impl DetectFix for FindToGlob {
    type FixInput<'a> = GlobSearch<'a>;

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
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'find' matching names");
                Some((
                    detection,
                    GlobSearch {
                        span: invocation.span,
                        search,
                    },
                ))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, glob: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = glob.search.to_glob();
        Some(Fix {
            explanation: format!("Search with '{replacement}'").into(),
            replacements: vec![Replacement::new(glob.span, replacement)],
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
