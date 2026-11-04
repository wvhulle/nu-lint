use nu_lint_analysis::{
    LintLevel,
    ast::span::SpanExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Span,
    ast::{Expr, Expression, PathMember},
};

struct CellPathIndexFixData {
    index_span: Span,
}

struct NumericIndexInfo {
    index_span: Span,
}

/// Returns info about the first non-optional numeric index in the cell path
fn find_first_numeric_index(members: &[PathMember]) -> Option<NumericIndexInfo> {
    members.iter().find_map(|m| match m {
        // Skip optional indices (with ?) as they return null instead of panicking
        PathMember::Int {
            span,
            optional: false,
            ..
        } => Some(NumericIndexInfo { index_span: *span }),
        PathMember::Int { optional: true, .. } | PathMember::String { .. } => None,
    })
}

fn check_cell_path_access(
    expr: &Expression,
    safe_context_spans: &[Span],
    explicit_optional: bool,
) -> Option<(Detection, CellPathIndexFixData)> {
    let Expr::FullCellPath(cell_path) = &expr.expr else {
        return None;
    };

    let index_info = find_first_numeric_index(&cell_path.tail)?;

    if expr.span.is_inside_any(safe_context_spans) {
        return None;
    }

    let (message, label) = if explicit_optional {
        (
            "Rewrite as `| get -o <index>` to avoid panic on empty list",
            "rewrite with `get -o`",
        )
    } else {
        (
            "Use optional access `?` to avoid panic on empty list",
            "add `?` for safe access",
        )
    };

    let violation =
        Detection::from_global_span(message, index_info.index_span).with_primary_label(label);

    let fix_data = CellPathIndexFixData {
        index_span: index_info.index_span,
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
             checking if the list is empty can cause a runtime panic. Use optional access with \
             `?` (e.g., $list.0?) which returns null instead of panicking. Alternatively, wrap in \
             a `try` block or check with `is-empty` first.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/navigating_structured_data.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let safe_context_spans = context.collect_command_spans(&["try", "if"]);
        let explicit_optional = context.config.explicit_optional_access;
        context.detect_with_fix_data(|expr, _ctx| {
            check_cell_path_access(expr, &safe_context_spans, explicit_optional)
                .into_iter()
                .collect()
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        if context.config.explicit_optional_access {
            // When explicit_optional_access is true, don't auto-fix with `?`
            // The user prefers `get --optional` which requires manual transformation
            return None;
        }

        let index_end = fix_data.index_span.end;

        Some(Fix {
            explanation: "Add `?` for safe optional access".into(),
            replacements: vec![Replacement::new(
                Span::new(index_end, index_end),
                "?".to_string(),
            )],
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
