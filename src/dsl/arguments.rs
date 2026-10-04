pub struct HttpHeader<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

pub fn http_header(text: &str) -> Option<HttpHeader<'_>> {
    let (name, value) = text.split_once(':')?;
    Some(HttpHeader {
        name: name.trim(),
        value: value.trim(),
    })
}

pub fn tail_start_line(count: &str) -> Option<u64> {
    count.strip_prefix('+')?.parse().ok()
}

pub fn is_shell_variable_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}
