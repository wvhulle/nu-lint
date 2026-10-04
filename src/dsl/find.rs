use nu_protocol::ast::ExternalArgument;

use crate::ast::{external::literal_content, string::quote_nu_string};

const GLOB_SPECIAL_CHARS: &[char] = &['*', '?', '[', ']', '{', '}'];
const NAME_CHARS_WITH_OTHER_GLOB_MEANING: &[char] = &['{', '}', '/'];
const RECURSIVE_WILDCARD: &str = "**";
const CURRENT_DIRECTORY: &str = ".";

#[derive(Clone, Copy)]
pub enum EntryType {
    Any,
    File,
    Directory,
}

#[derive(Clone, Copy)]
pub struct FileSearch<'a> {
    directory: &'a str,
    name: Option<&'a str>,
    entry_type: EntryType,
}

impl<'a> FileSearch<'a> {
    pub fn parse(args: &'a [ExternalArgument]) -> Option<Self> {
        let words: Option<Vec<&'a str>> = args
            .iter()
            .map(|argument| match argument {
                ExternalArgument::Regular(expr) => literal_content(expr),
                ExternalArgument::Spread(_) => None,
            })
            .collect();
        let words = words?;
        let (directory, predicates) = match words.split_first() {
            Some((first, rest)) if !first.starts_with('-') => (*first, rest),
            _ => (CURRENT_DIRECTORY, words.as_slice()),
        };
        if directory.contains(GLOB_SPECIAL_CHARS) {
            return None;
        }
        let search = Self {
            directory: directory.trim_end_matches('/'),
            name: None,
            entry_type: EntryType::Any,
        };
        predicates.chunks(2).try_fold(search, Self::with_predicate)
    }

    fn with_predicate(self, predicate: &[&'a str]) -> Option<Self> {
        match predicate {
            ["-name", name] if self.name.is_none() && name_matches_like_glob(name) => Some(Self {
                name: Some(name),
                ..self
            }),
            ["-type", "f"] => Some(Self {
                entry_type: EntryType::File,
                ..self
            }),
            ["-type", "d"] => Some(Self {
                entry_type: EntryType::Directory,
                ..self
            }),
            _ => None,
        }
    }

    pub fn to_glob(self) -> String {
        let directory = self.directory;
        let pattern = self.name.map_or_else(
            || format!("{directory}/{RECURSIVE_WILDCARD}"),
            |name| format!("{directory}/{RECURSIVE_WILDCARD}/{name}"),
        );
        let filter = match self.entry_type {
            EntryType::Any => "",
            EntryType::File => " --no-dir --no-symlink",
            EntryType::Directory => " --no-file --no-symlink",
        };
        format!("glob {}{filter}", quote_nu_string(&pattern))
    }
}

fn name_matches_like_glob(name: &str) -> bool {
    !name.contains(NAME_CHARS_WITH_OTHER_GLOB_MEANING) && !name.contains(RECURSIVE_WILDCARD)
}
