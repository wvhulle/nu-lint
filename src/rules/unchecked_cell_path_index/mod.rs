use nu_protocol::{
    Span,
    ast::{Expr, Expression, PathMember},
};

use crate::{
    LintLevel,
    ast::span::SpanExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct CellPathIndexFixData {
    full_span: Span,
    base_span: Span,
    index_value: usize,
}

struct NumericIndexInfo {
    member_idx: usize,
    index_value: usize,
    index_span: Span,
}

/// Returns info about the first numeric index in the cell path
fn find_first_numeric_index(members: &[PathMember]) -> Option<NumericIndexInfo> {
    members
        .iter()
        .enumerate()
        .find_map(|(member_idx, m)| match m {
            PathMember::Int { val, span, .. } => Some(NumericIndexInfo {
                member_idx,
                index_value: *val,
                index_span: *span,
            }),
            PathMember::String { .. } => None,
        })
}

fn check_cell_path_access(
    expr: &Expression,
    safe_context_spans: &[Span],
) -> Option<(Detection, CellPathIndexFixData)> {
    let Expr::FullCellPath(cell_path) = &expr.expr else {
        return None;
    };

    let index_info = find_first_numeric_index(&cell_path.tail)?;

    if expr.span.is_inside_any(safe_context_spans) {
        return None;
    }

    // Calculate the span of the base expression (everything before the first
    // numeric index) If member_idx is 0, the base is just the head
    let base_span = if index_info.member_idx == 0 {
        cell_path.head.span
    } else {
        // Include head and members up to (but not including) the numeric index
        let last_string_member = &cell_path.tail[index_info.member_idx - 1];
        let last_span = match last_string_member {
            PathMember::String { span, .. } | PathMember::Int { span, .. } => *span,
        };
        Span::new(cell_path.head.span.start, last_span.end)
    };

    let violation = Detection::from_global_span(
        "List index access without bounds check may panic if list is empty",
        index_info.index_span,
    )
    .with_primary_label("unchecked index access");

    let fix_data = CellPathIndexFixData {
        full_span: expr.span,
        base_span,
        index_value: index_info.index_value,
    };

    Some((violation, fix_data))
}

struct UncheckedCellPathIndex;

impl DetectFix for UncheckedCellPathIndex {
    type FixInput<'a> = CellPathIndexFixData;

    fn id(&self) -> &'static str {
        "unchecked_cell_path_index"
    }

    fn short_description(&self) -> &'static str {
        "Cell path numeric index access may panic on empty lists"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Accessing list elements by numeric index using cell paths (e.g., $list.0) without \
             checking if the list is empty can cause a runtime panic. Consider wrapping the \
             access in a 'try' block or checking with 'is-empty' first. Alternatively, use 'get \
             -o 0' for safe optional access.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/navigating_structured_data.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let safe_context_spans = context.collect_command_spans(&["try", "if"]);
        context.detect_with_fix_data(|expr, _ctx| {
            check_cell_path_access(expr, &safe_context_spans)
                .into_iter()
                .collect()
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let base_text = context.span_text(fix_data.base_span);
        let replacement = format!("{base_text} | get -o {}", fix_data.index_value);

        Some(Fix {
            explanation: "Convert to safe 'get -o' access".into(),
            replacements: vec![Replacement::new(fix_data.full_span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &UncheckedCellPathIndex;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
