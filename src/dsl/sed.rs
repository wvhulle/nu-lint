const BASIC_REGEX_SPECIAL_CHARS: &[char] = &['.', '*', '[', ']', '^', '$', '\\', '\n'];
const REPLACEMENT_SPECIAL_CHARS: &[char] = &['&', '\\', '\n'];
const GLOBAL_FLAG: &str = "g";

pub struct Substitution<'a> {
    pub find: &'a str,
    pub replace: &'a str,
}

pub fn global_literal_substitution(script: &str) -> Option<Substitution<'_>> {
    let body = script.strip_prefix('s')?;
    let delimiter = body.chars().next().filter(|c| !c.is_alphanumeric())?;
    let mut parts = body[delimiter.len_utf8()..].split(delimiter);
    let find = parts.next()?;
    let replace = parts.next()?;
    let flags = parts.next()?;
    let is_literal = !find.is_empty()
        && !find.contains(BASIC_REGEX_SPECIAL_CHARS)
        && !replace.contains(REPLACEMENT_SPECIAL_CHARS);
    (parts.next().is_none() && flags == GLOBAL_FLAG && is_literal)
        .then_some(Substitution { find, replace })
}
