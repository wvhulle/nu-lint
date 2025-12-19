use jaq_core::{
    load::{
        self,
        lex::{Lexer, StrPart},
        parse::{Parser, Term},
    },
    path::{self, Opt, Part},
};
use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

/// Extract field name from a path like .field
fn extract_field_from_path<'a>(term: &'a Term<&str>) -> Option<&'a str> {
    if let Term::Path(inner, path) = term
        && matches!(**inner, Term::Id)
        && path.0.len() == 1
        && let Part::Index(Term::Str(_, parts)) = &path.0[0].0
        && parts.len() == 1
        && let StrPart::Str(field) = &parts[0]
    {
        Some(field)
    } else {
        None
    }
}

/// Extract field name from a Part if it's a simple string index
fn extract_field_from_part<'a>(part: &(Part<Term<&'a str>>, Opt)) -> Option<&'a str> {
    if let Part::Index(Term::Str(_, parts)) = &part.0
        && parts.len() == 1
        && let load::lex::StrPart::Str(field) = &parts[0]
    {
        Some(*field)
    } else {
        None
    }
}

/// Parse a jq filter string into an AST using jaq-core
fn parse_jq_filter(filter_str: &str) -> Option<Term<&str>> {
    // Remove surrounding quotes if present
    let trimmed = filter_str.trim();

    // Safety check for too-short strings
    if trimmed.len() < 2 {
        return None;
    }

    let filter_content = if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        || (trimmed.starts_with('"') && trimmed.ends_with('"'))
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    // Parse using jaq-core
    let tokens = Lexer::new(filter_content).lex().ok()?;
    Parser::new(&tokens).parse(Parser::term).ok()
}

/// Convert a jq Term AST to equivalent Nushell command
fn jq_term_to_nushell(term: &Term<&str>, has_file: bool) -> Option<String> {
    let wrap_with_open = |cmd: &str| -> String {
        if has_file {
            format!("open $file | from json | {cmd}")
        } else {
            cmd.to_string()
        }
    };

    match term {
        // Simple function calls like length, keys, add, etc.
        Term::Call(name, args) if args.is_empty() => convert_simple_call(name, &wrap_with_open),

        // Functions with arguments: map(.field), select(.active), group_by(.category),
        // sort_by(.field)
        Term::Call(name, args) if args.len() == 1 => {
            convert_call_with_arg(name, &args[0], &wrap_with_open)
        }

        // Path expressions: .[0], .[-1], .[], .field, .users[], .database.host
        Term::Path(inner, path_parts) if matches!(**inner, Term::Id) => {
            convert_path_expression(path_parts, &wrap_with_open)
        }

        // Pipe expressions: .users[] | .name
        Term::Pipe(left, _, right) => {
            let left_nu = jq_term_to_nushell(left, has_file)?;
            let right_nu = jq_term_to_nushell(right, false)?;
            Some(format!("{left_nu} | {right_nu}"))
        }

        _ => None,
    }
}

/// Convert simple jq function calls (no arguments)
fn convert_simple_call<F>(name: &str, wrap_with_open: &F) -> Option<String>
where
    F: Fn(&str) -> String,
{
    match name {
        "length" => Some(wrap_with_open("length")),
        "keys" => Some(wrap_with_open("columns")),
        "type" => Some(wrap_with_open("describe")),
        "empty" => Some("null".to_string()),
        "not" => Some(wrap_with_open("not $in")),
        "flatten" => Some(wrap_with_open("flatten")),
        "add" => Some(wrap_with_open("math sum")),
        "min" => Some(wrap_with_open("math min")),
        "max" => Some(wrap_with_open("math max")),
        "sort" => Some(wrap_with_open("sort")),
        "unique" => Some(wrap_with_open("uniq")),
        "reverse" => Some(wrap_with_open("reverse")),
        _ => None,
    }
}

/// Convert jq function calls with one argument
fn convert_call_with_arg<F>(name: &str, arg: &Term<&str>, wrap_with_open: &F) -> Option<String>
where
    F: Fn(&str) -> String,
{
    match name {
        "map" => {
            // map(.field) -> get field
            extract_field_from_path(arg).map(|field| wrap_with_open(&format!("get {field}")))
        }
        "select" => {
            // select(.field) -> where field
            // Only handle simple field access, not complex conditions
            extract_field_from_path(arg).map(|field| wrap_with_open(&format!("where {field}")))
        }
        "group_by" => {
            // group_by(.field) -> group-by field
            extract_field_from_path(arg).map(|field| wrap_with_open(&format!("group-by {field}")))
        }
        "sort_by" => {
            // sort_by(.field) -> sort-by field
            extract_field_from_path(arg).map(|field| wrap_with_open(&format!("sort-by {field}")))
        }
        _ => None,
    }
}

/// Convert jq path expressions to Nushell
fn convert_path_expression<F>(
    path_parts: &path::Path<Term<&str>>,
    wrap_with_open: &F,
) -> Option<String>
where
    F: Fn(&str) -> String,
{
    match path_parts.0.len() {
        1 => convert_single_part_path(&path_parts.0[0].0, wrap_with_open),
        2 => convert_two_part_path(&path_parts.0[0].0, &path_parts.0[1].0, wrap_with_open),
        _ => convert_multi_part_path(path_parts, wrap_with_open),
    }
}

/// Convert single-part path expressions
fn convert_single_part_path<F>(part: &Part<Term<&str>>, wrap_with_open: &F) -> Option<String>
where
    F: Fn(&str) -> String,
{
    match part {
        // Positive numeric index: .[0], .[1], etc.
        Part::Index(Term::Num(n)) => Some(wrap_with_open(&format!("get {n}"))),
        // Negative numeric index: .[-1] is parsed as Neg(Num("1"))
        Part::Index(Term::Neg(inner_term)) => match &**inner_term {
            Term::Num(n) if n == &"1" => Some(wrap_with_open("last")),
            Term::Num(n) => Some(wrap_with_open(&format!("get -{n}"))),
            _ => None,
        },
        // Array iteration: .[]
        Part::Range(None, None) => Some(wrap_with_open("each")),
        // Field access: .field is Index(Str(...))
        Part::Index(Term::Str(_, parts)) if parts.len() == 1 => {
            if let load::lex::StrPart::Str(field) = &parts[0] {
                Some(wrap_with_open(&format!("get {field}")))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Convert two-part path expressions
fn convert_two_part_path<F>(
    part1: &Part<Term<&str>>,
    part2: &Part<Term<&str>>,
    wrap_with_open: &F,
) -> Option<String>
where
    F: Fn(&str) -> String,
{
    // .users[] pattern: field access followed by array iteration
    if let Part::Index(Term::Str(_, parts)) = part1
        && parts.len() == 1
        && let load::lex::StrPart::Str(field) = &parts[0]
        && matches!(part2, Part::Range(None, None))
    {
        return Some(wrap_with_open(&format!("get {field} | each")));
    }

    // .database.host pattern: multiple field accesses
    if let Part::Index(Term::Str(_, parts1)) = part1
        && let Part::Index(Term::Str(_, parts2)) = part2
        && parts1.len() == 1
        && parts2.len() == 1
        && let load::lex::StrPart::Str(field1) = &parts1[0]
        && let load::lex::StrPart::Str(field2) = &parts2[0]
    {
        return Some(wrap_with_open(&format!("get {field1}.{field2}")));
    }

    None
}

/// Convert multi-part path expressions (3+ parts)
fn convert_multi_part_path<F>(
    path_parts: &path::Path<Term<&str>>,
    wrap_with_open: &F,
) -> Option<String>
where
    F: Fn(&str) -> String,
{
    // Try to extract all field names for longer paths like .a.b.c
    let fields: Vec<_> = path_parts
        .0
        .iter()
        .filter_map(extract_field_from_part)
        .collect();

    (fields.len() == path_parts.0.len() && !fields.is_empty())
        .then(|| wrap_with_open(&format!("get {}", fields.join("."))))
}

/// Simple jq operations that have direct Nushell equivalents
const SIMPLE_JQ_OPS: &[&str] = &[
    "'length'",
    "'keys'",
    "'type'",
    "'empty'",
    "'not'",
    "'flatten'",
    "'add'",
    "'min'",
    "'max'",
    "'sort'",
    "'unique'",
];

const NOTE: &str =
    "Use built-in Nushell commands for simple operations - they're faster and more idiomatic";

fn format_jq_replacement(filter: &str, file_arg: Option<&str>) -> String {
    // Try to parse the jq filter using jaq-syn
    if let Some(term) = parse_jq_filter(filter)
        && let Some(nu_cmd) = jq_term_to_nushell(&term, file_arg.is_some())
    {
        return nu_cmd;
    }

    // Fallback to string-based matching for backward compatibility
    let with_file = |cmd: &str| {
        file_arg.map_or_else(
            || cmd.to_string(),
            |file| format!("open {file} | from json | {cmd}"),
        )
    };

    match filter {
        "'length'" => with_file("length"),
        "'keys'" => with_file("columns"),
        "'type'" => with_file("describe"),
        "'empty'" => "null".to_string(),
        "'not'" => "not".to_string(),
        "'flatten'" => "flatten".to_string(),
        "'add'" => "math sum".to_string(),
        "'min'" => "math min".to_string(),
        "'max'" => "math max".to_string(),
        "'sort'" => "sort".to_string(),
        "'unique'" => "uniq".to_string(),
        _ if filter.starts_with("'.[") && filter.ends_with("]'") => {
            let index = &filter[3..filter.len() - 2];
            format!("get {index}")
        }
        _ => file_arg.map_or_else(
            || "from json".to_string(),
            |file| format!("open {file} | from json"),
        ),
    }
}

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text: Vec<&str> = external_args_slices(args, context).collect();

    let new_text = if args_text.is_empty() {
        // Without explicit filter or file, suggest decoding JSON from input
        "from json".to_string()
    } else {
        let filter = args_text[0];
        let file_arg = args_text.get(1).copied();
        format_jq_replacement(filter, file_arg)
    };

    Fix::with_explanation(
        "Replace simple jq operation with built-in Nushell command",
        vec![Replacement::new(expr_span, new_text)],
    )
}

/// Check if a jq command contains simple operations
fn contains_simple_jq_op(source_text: &str) -> bool {
    // Extract the filter argument from the jq command
    // Handle both single-line and multi-line jq filters
    // Skip "jq" or "^jq", skip flags (starting with -), then extract the quoted
    // filter

    // Find the start of the filter (first quote after jq and any flags)
    let quote_start = source_text.find(['\'', '"']);

    if let Some(start_pos) = quote_start {
        let quote_char = source_text.chars().nth(start_pos).unwrap();
        // Find the matching closing quote
        if let Some(end_pos) = source_text[start_pos + 1..].rfind(quote_char) {
            let filter_str = &source_text[start_pos..=start_pos + 1 + end_pos];

            // Try to parse and see if we can convert it
            if let Some(term) = parse_jq_filter(filter_str) {
                return jq_term_to_nushell(&term, false).is_some();
            }
        }
    }

    // Fallback to string-based matching
    SIMPLE_JQ_OPS.iter().any(|op| source_text.contains(op))
        || (source_text.contains("'.[") && source_text.contains("]'"))
}

fn check(context: &LintContext) -> Vec<Violation> {
    // Detect jq commands that have direct Nushell equivalents
    let violations = detect_external_commands(context, "jq", NOTE, Some(build_fix));

    // Filter to only show violations for jq operations we can convert
    violations
        .into_iter()
        .filter(|violation| {
            let source_text = std::str::from_utf8(context.working_set.get_span_contents(violation.span))
                .unwrap_or("");
            contains_simple_jq_op(source_text)
        })
        .collect()
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_nushell_over_jq",
        "Prefer Nushell built-ins over jq for data operations that have direct equivalents",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/from_json.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
