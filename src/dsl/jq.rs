//! jq filter to Nushell command conversion
//!
//! Converts simple jq filters to equivalent Nushell pipelines.
//! Used by lint rules to suggest native Nushell alternatives to jq.

use std::borrow::Cow;

use jaq_core::{
    load::{
        lex::{Lexer, StrPart},
        parse::{Parser, Term},
    },
    path::{self, Opt, Part},
};
use nu_protocol::{
    Span,
    ast::{Expr, Expression},
};

use super::ConversionContext;
use crate::{ast::string::cell_path_member_needs_quotes, context::LintContext};

/// A static field path extracted from a jq filter (e.g., `.a.b.c`).
#[derive(Debug, Clone)]
pub struct FieldPath {
    segments: Vec<String>,
}

impl FieldPath {
    fn from_segments(segments: Vec<String>) -> Option<Self> {
        if segments.is_empty() {
            None
        } else {
            Some(Self { segments })
        }
    }

    fn as_dotted(&self) -> String {
        self.segments
            .iter()
            .map(|s| maybe_quote_field(s))
            .collect::<Vec<_>>()
            .join(".")
    }
}

/// Quote a field name if it contains special characters.
fn maybe_quote_field(s: &str) -> String {
    if cell_path_member_needs_quotes(s) {
        format!("\"{s}\"")
    } else {
        s.to_string()
    }
}

/// A numeric index value from a jq filter.
#[derive(Debug, Clone, Copy)]
pub enum IndexValue {
    Positive(u64),
    Negative(u64),
}

impl IndexValue {
    fn parse(s: &str) -> Option<Self> {
        s.parse().ok().map(Self::Positive)
    }

    const fn negative(n: u64) -> Self {
        Self::Negative(n)
    }

    const fn is_last(&self) -> bool {
        matches!(self, Self::Negative(1))
    }
}

/// Semantic representation of a Nu command equivalent to a jq filter.
///
/// Text generation happens in `format()` using `LintContext::span_text()` for
/// dynamic parts.
#[derive(Debug, Clone)]
pub enum NuEquivalent {
    Command(&'static str),
    GetPath(FieldPath),
    GetThenIterate(FieldPath),
    GetIndex(IndexValue),
    CommandWithField { nu_cmd: &'static str, field: String },
    DynamicGet { var_span: Span },
    DynamicGetWithPrefix { prefix: String, var_span: Span },
    DynamicIndex { var_span: Span },
    FieldThenDynamicIndex { field: String, var_span: Span },
    Pipe { left: Box<Self>, right: Box<Self> },
}

impl NuEquivalent {
    pub fn format(&self, ctx: &ConversionContext, lint_ctx: &LintContext) -> Cow<'static, str> {
        let cmd = self.to_nu_command(lint_ctx);
        ctx.wrap_str(&cmd)
    }

    fn to_nu_command(&self, lint_ctx: &LintContext) -> String {
        match self {
            Self::Command(cmd) => (*cmd).to_string(),
            Self::GetPath(path) => format!("get {}", path.as_dotted()),
            Self::GetThenIterate(path) => format!("get {} | each", path.as_dotted()),
            Self::GetIndex(IndexValue::Positive(n)) => format!("get {n}"),
            Self::GetIndex(IndexValue::Negative(n)) => format!("get -{n}"),
            Self::CommandWithField { nu_cmd, field } => {
                format!("{nu_cmd} {}", maybe_quote_field(field))
            }
            Self::DynamicGet { var_span } | Self::DynamicIndex { var_span } => {
                format!("get {}", lint_ctx.span_text(*var_span))
            }
            Self::DynamicGetWithPrefix { prefix, var_span } => {
                format!(
                    "get {}.{}",
                    maybe_quote_field(prefix),
                    lint_ctx.span_text(*var_span)
                )
            }
            Self::FieldThenDynamicIndex { field, var_span } => {
                format!(
                    "get {} | get {}",
                    maybe_quote_field(field),
                    lint_ctx.span_text(*var_span)
                )
            }
            Self::Pipe { left, right } => {
                format!(
                    "{} | {}",
                    left.to_nu_command(lint_ctx),
                    right.to_nu_command(lint_ctx)
                )
            }
        }
    }
}

/// Parse and convert a jq filter string to Nushell.
/// Returns `None` if the filter cannot be converted.
pub fn convert(filter: &str) -> Option<NuEquivalent> {
    let term = parse_filter(filter)?;
    convert_term(&term)
}

fn parse_filter(filter: &str) -> Option<Term<&str>> {
    let content = filter
        .trim()
        .strip_prefix(['\'', '"'])
        .and_then(|s| s.strip_suffix(['\'', '"']))
        .unwrap_or(filter.trim());
    if content.is_empty() {
        return None;
    }
    let tokens = Lexer::new(content).lex().ok()?;
    Parser::new(&tokens).parse(Parser::term).ok()
}

fn convert_term(term: &Term<&str>) -> Option<NuEquivalent> {
    match term {
        Term::Call(name, args) if args.is_empty() => convert_builtin(name),
        Term::Call(name, args) if args.len() == 1 => convert_call_with_arg(name, &args[0]),
        Term::Path(inner, path) if matches!(**inner, Term::Id) => convert_path(path),
        Term::Pipe(left, _, right) => {
            let left_conv = convert_term(left)?;
            let right_conv = convert_term(right)?;
            Some(NuEquivalent::Pipe {
                left: Box::new(left_conv),
                right: Box::new(right_conv),
            })
        }
        _ => None,
    }
}

fn convert_builtin(name: &str) -> Option<NuEquivalent> {
    let cmd: &'static str = match name {
        "type" => "describe",
        "empty" => "null",
        "not" => "not $in",
        "flatten" => "flatten",
        "add" => "math sum",
        "min" => "math min",
        "max" => "math max",
        "sort" => "sort",
        "unique" => "uniq",
        "reverse" => "reverse",
        _ => return None,
    };
    Some(NuEquivalent::Command(cmd))
}

fn convert_call_with_arg(name: &str, arg: &Term<&str>) -> Option<NuEquivalent> {
    let field = extract_single_field_from_term(arg)?;
    let nu_cmd: &'static str = match name {
        "map" => "get",
        "select" => "where",
        "group_by" => "group-by",
        "sort_by" => "sort-by",
        _ => return None,
    };
    Some(NuEquivalent::CommandWithField {
        nu_cmd,
        field: field.to_string(),
    })
}

fn convert_path(path: &path::Path<Term<&str>>) -> Option<NuEquivalent> {
    let parts = &path.0;

    match parts.as_slice() {
        [(Part::Index(Term::Num(n)), _)] => {
            let idx = IndexValue::parse(n)?;
            Some(NuEquivalent::GetIndex(idx))
        }
        [(Part::Index(Term::Neg(inner)), _)] => match &**inner {
            Term::Num(n) => {
                let num: u64 = n.parse().ok()?;
                let idx = IndexValue::negative(num);
                if idx.is_last() {
                    Some(NuEquivalent::Command("last"))
                } else {
                    Some(NuEquivalent::GetIndex(idx))
                }
            }
            _ => None,
        },
        [(Part::Range(None, None), _)] => Some(NuEquivalent::Command("each")),
        _ if is_all_field_access(parts) => {
            let segments = extract_field_names(parts)?;
            let path = FieldPath::from_segments(segments)?;
            Some(NuEquivalent::GetPath(path))
        }
        [field_parts @ .., (Part::Range(None, None), _)] if is_all_field_access(field_parts) => {
            let segments = extract_field_names(field_parts)?;
            let path = FieldPath::from_segments(segments)?;
            Some(NuEquivalent::GetThenIterate(path))
        }
        _ => None,
    }
}

fn is_all_field_access(parts: &[(Part<Term<&str>>, Opt)]) -> bool {
    !parts.is_empty() && parts.iter().all(|(p, _)| extract_single_field(p).is_some())
}

fn extract_field_names(parts: &[(Part<Term<&str>>, Opt)]) -> Option<Vec<String>> {
    parts
        .iter()
        .map(|(part, _)| extract_single_field(part).map(String::from))
        .collect()
}

fn extract_single_field<'a>(part: &Part<Term<&'a str>>) -> Option<&'a str> {
    if let Part::Index(Term::Str(_, str_parts)) = part
        && str_parts.len() == 1
        && let StrPart::Str(field) = &str_parts[0]
    {
        Some(*field)
    } else {
        None
    }
}

fn extract_single_field_from_term<'a>(term: &'a Term<&str>) -> Option<&'a str> {
    if let Term::Path(inner, path) = term
        && matches!(**inner, Term::Id)
        && path.0.len() == 1
    {
        extract_single_field(&path.0[0].0)
    } else {
        None
    }
}

/// Convert interpolated jq filter to Nushell.
/// Handles patterns like `$".($field)"` → `get $field`
pub fn convert_interpolation(exprs: &[Expression], lint_ctx: &LintContext) -> Option<NuEquivalent> {
    match exprs {
        [dot, var_expr] if is_dot_string(dot) => {
            let var_span = extract_var_span(var_expr, lint_ctx)?;
            Some(NuEquivalent::DynamicGet { var_span })
        }
        [prefix, var_expr] if is_field_prefix(prefix) => {
            let field = extract_field_prefix(prefix)?.to_string();
            let var_span = extract_var_span(var_expr, lint_ctx)?;
            Some(NuEquivalent::DynamicGetWithPrefix {
                prefix: field,
                var_span,
            })
        }
        [open, idx_expr, close] if is_index_open(open) && is_index_close(close) => {
            let var_span = extract_var_span(idx_expr, lint_ctx)?;
            Some(NuEquivalent::DynamicIndex { var_span })
        }
        [prefix, idx_expr, close] if is_field_index_prefix(prefix) && is_index_close(close) => {
            let field = extract_field_index_prefix(prefix)?.to_string();
            let var_span = extract_var_span(idx_expr, lint_ctx)?;
            Some(NuEquivalent::FieldThenDynamicIndex { field, var_span })
        }
        _ => None,
    }
}

fn is_dot_string(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::String(s) if s == ".")
}

fn is_field_prefix(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::String(s) if s.starts_with('.') && s.ends_with('.') && s.len() > 2)
}

fn extract_field_prefix(expr: &Expression) -> Option<&str> {
    if let Expr::String(s) = &expr.expr {
        s.strip_prefix('.')?.strip_suffix('.')
    } else {
        None
    }
}

fn is_index_open(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::String(s) if s == ".[")
}

fn is_index_close(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::String(s) if s == "]")
}

fn is_field_index_prefix(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::String(s) if s.starts_with('.') && s.ends_with('[') && s.len() > 2)
}

fn extract_field_index_prefix(expr: &Expression) -> Option<&str> {
    if let Expr::String(s) = &expr.expr {
        s.strip_prefix('.')?.strip_suffix('[')
    } else {
        None
    }
}

fn extract_var_span(expr: &Expression, ctx: &LintContext) -> Option<Span> {
    let inner = match &expr.expr {
        Expr::FullCellPath(fcp) if fcp.tail.is_empty() => &fcp.head,
        _ => expr,
    };

    match &inner.expr {
        Expr::Subexpression(block_id) => {
            let block = ctx.working_set.get_block(*block_id);
            let pipeline = block.pipelines.first()?;
            let elem = pipeline.elements.first()?;
            match &elem.expr.expr {
                Expr::Var(_) | Expr::FullCellPath(_) => Some(elem.expr.span),
                _ => None,
            }
        }
        _ => None,
    }
}
