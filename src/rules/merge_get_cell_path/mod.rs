use nu_protocol::ast::{Expr, Expression, PathMember, Pipeline};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

fn extract_cell_path_members(expr: &Expression) -> Option<Vec<PathMember>> {
    if let Expr::CellPath(cell_path) = &expr.expr {
        Some(cell_path.members.clone())
    } else {
        None
    }
}

fn is_get_call(expr: &Expression, context: &LintContext) -> bool {
    matches!(&expr.expr, Expr::Call(call) if call.is_call_to_command("get", context))
}

fn combine_cell_path_members(members: &[Vec<PathMember>]) -> Vec<PathMember> {
    members.iter().flatten().cloned().collect()
}

fn format_path_member(member: &PathMember) -> String {
    match member {
        PathMember::String { val, optional, .. } => {
            let prefix = if *optional { "?." } else { "" };
            if val.contains(' ') || val.parse::<i64>().is_ok() {
                format!("{prefix}\"{val}\"")
            } else {
                format!("{prefix}{val}")
            }
        }
        PathMember::Int { val, optional, .. } => {
            let prefix = if *optional { "?." } else { "" };
            format!("{prefix}{val}")
        }
    }
}

fn format_cell_path(members: &[PathMember]) -> String {
    members
        .iter()
        .map(format_path_member)
        .collect::<Vec<_>>()
        .join(".")
}

const fn get_command_label(idx: usize, start_idx: usize, end_idx: usize) -> &'static str {
    if idx == start_idx {
        "First 'get' command"
    } else if idx == end_idx {
        "Last 'get' command"
    } else {
        "Intermediate 'get' command"
    }
}

fn find_consecutive_gets(pipeline: &Pipeline, context: &LintContext) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start_idx = None;

    for (idx, element) in pipeline.elements.iter().enumerate() {
        if is_get_call(&element.expr, context) {
            start_idx.get_or_insert(idx);
        } else if let Some(start) = start_idx.take()
            && idx - start >= 2
        {
            ranges.push((start, idx - 1));
        }
    }

    if let Some(start) = start_idx {
        let end = pipeline.elements.len() - 1;
        if end > start {
            ranges.push((start, end));
        }
    }

    ranges
}

fn collect_cell_path_members(
    pipeline: &Pipeline,
    start_idx: usize,
    end_idx: usize,
) -> Option<Vec<Vec<PathMember>>> {
    (start_idx..=end_idx)
        .map(|idx| {
            let element = &pipeline.elements[idx];
            let Expr::Call(call) = &element.expr.expr else {
                return None;
            };
            let arg = call.get_first_positional_arg()?;
            extract_cell_path_members(arg)
        })
        .collect()
}

fn generate_fix(pipeline: &Pipeline, start_idx: usize, end_idx: usize) -> Option<Fix> {
    let all_members = collect_cell_path_members(pipeline, start_idx, end_idx)?;
    let combined_members = combine_cell_path_members(&all_members);
    let combined_path = format_cell_path(&combined_members);

    let start_span = pipeline.elements[start_idx].expr.span;
    let end_span = pipeline.elements[end_idx].expr.span;
    let full_span = nu_protocol::Span::new(start_span.start, end_span.end);

    let replacement_text = format!("get {combined_path}");

    Some(Fix::with_explanation(
        format!("Combine into single cell path: {replacement_text}"),
        vec![Replacement::new(full_span, replacement_text)],
    ))
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    for pipeline in &context.ast.pipelines {
        let consecutive_ranges = find_consecutive_gets(pipeline, context);

        for (start_idx, end_idx) in consecutive_ranges {
            let start_span = pipeline.elements[start_idx].expr.span;
            let end_span = pipeline.elements[end_idx].expr.span;
            let full_span = nu_protocol::Span::new(start_span.start, end_span.end);

            let num_gets = end_idx - start_idx + 1;
            let message =
                format!("Use combined cell path instead of {num_gets} chained 'get' commands");

            let fix = generate_fix(pipeline, start_idx, end_idx);

            let mut violation = Violation::new(message, full_span)
                .with_help("Combine into single 'get' with dot-separated cell path");

            for idx in start_idx..=end_idx {
                let span = pipeline.elements[idx].expr.span;
                let label = get_command_label(idx, start_idx, end_idx);
                violation = violation.with_extra_label(label, span);
            }

            if let Some(f) = fix {
                violation = violation.with_fix(f);
            }

            violations.push(violation);
        }
    }

    violations
}

pub const RULE: Rule = Rule::new(
    "merge_get_cell_path",
    "Prefer combined cell paths over chained 'get' commands",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/book/navigating_structured_data.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
