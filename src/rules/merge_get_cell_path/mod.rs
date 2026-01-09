use nu_protocol::{
    Span,
    ast::{Expr, Expression, PathMember, Pipeline},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, string::cell_path_member_needs_quotes},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Semantic fix data: stores the spans and cell path members for generating fix
pub struct FixData {
    full_span: Span,
    /// Combined cell path members from all get commands
    combined_members: Vec<PathMember>,
}

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

fn combine_cell_path_members(member_groups: &[Vec<PathMember>]) -> Vec<PathMember> {
    member_groups.iter().flatten().cloned().collect()
}

const fn format_optional_prefix(optional: bool) -> &'static str {
    if optional { "?." } else { "" }
}

fn format_path_member(member: &PathMember) -> String {
    match member {
        PathMember::String { val, optional, .. } => {
            let prefix = format_optional_prefix(*optional);
            if cell_path_member_needs_quotes(val) {
                format!("{prefix}\"{val}\"")
            } else {
                format!("{prefix}{val}")
            }
        }
        PathMember::Int { val, optional, .. } => {
            let prefix = format_optional_prefix(*optional);
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

const MIN_CONSECUTIVE_GETS: usize = 2;

fn find_consecutive_gets(pipeline: &Pipeline, context: &LintContext) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut current_range_start = None;

    for (idx, element) in pipeline.elements.iter().enumerate() {
        if is_get_call(&element.expr, context) {
            current_range_start.get_or_insert(idx);
        } else if let Some(start) = current_range_start.take() {
            let range_length = idx - start;
            if range_length >= MIN_CONSECUTIVE_GETS {
                ranges.push((start, idx - 1));
            }
        }
    }

    if let Some(start) = current_range_start {
        let last_idx = pipeline.elements.len() - 1;
        let range_length = last_idx - start + 1;
        if range_length >= MIN_CONSECUTIVE_GETS {
            ranges.push((start, last_idx));
        }
    }

    ranges
}

fn collect_cell_path_members_from_pipeline(
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

fn calculate_full_span(pipeline: &Pipeline, start_idx: usize, end_idx: usize) -> Span {
    let start_span = pipeline.elements[start_idx].expr.span;
    let end_span = pipeline.elements[end_idx].expr.span;
    Span::new(start_span.start, end_span.end)
}

fn create_violation_for_range(pipeline: &Pipeline, start_idx: usize, end_idx: usize) -> Detection {
    let full_span = calculate_full_span(pipeline, start_idx, end_idx);
    let num_gets = end_idx - start_idx + 1;
    let message = format!("Use combined cell path instead of {num_gets} chained 'get' commands");

    let mut violation = Detection::from_global_span(message, full_span);

    for idx in start_idx..=end_idx {
        let span = pipeline.elements[idx].expr.span;
        let label = get_command_label(idx, start_idx, end_idx);
        violation = violation.with_extra_label(label, span);
    }

    violation
}

struct MergeGetCellPath;

impl DetectFix for MergeGetCellPath {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "merge_get_cell_path"
    }

    fn short_description(&self) -> &'static str {
        "Prefer combined cell paths over chained 'get' commands"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some("Combine into single 'get' with dot-separated cell path")
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/navigating_structured_data.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .ast
            .pipelines
            .iter()
            .flat_map(|pipeline| {
                find_consecutive_gets(pipeline, context)
                    .into_iter()
                    .filter_map(|(start_idx, end_idx)| {
                        let violation = create_violation_for_range(pipeline, start_idx, end_idx);

                        let member_groups =
                            collect_cell_path_members_from_pipeline(pipeline, start_idx, end_idx)?;
                        let combined_members = combine_cell_path_members(&member_groups);

                        let fix_data = FixData {
                            full_span: calculate_full_span(pipeline, start_idx, end_idx),
                            combined_members,
                        };

                        Some((violation, fix_data))
                    })
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let combined_path = format_cell_path(&fix_data.combined_members);
        let replacement_text = format!("get {combined_path}");

        Some(Fix::with_explanation(
            format!("Combine into single cell path: {replacement_text}"),
            vec![Replacement::new(fix_data.full_span, replacement_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &MergeGetCellPath;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
