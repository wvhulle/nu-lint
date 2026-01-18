use nu_protocol::{
    Span,
    ast::{Expr, Expression, PathMember, PipelineElement},
};

use crate::{
    LintLevel,
    ast::{
        call::CallExt,
        pipeline::{ClusterConfig, PipelineExt},
        string::cell_path_member_needs_quotes,
    },
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

/// Extract cell path members from a pipeline element's get call
fn extract_members_from_get_call(element: &PipelineElement) -> Option<Vec<PathMember>> {
    let Expr::Call(call) = &element.expr.expr else {
        return None;
    };
    let arg = call.get_first_positional_arg()?;
    extract_cell_path_members(arg)
}

struct MergeGetCellPath;

impl DetectFix for MergeGetCellPath {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "merge_get_cell_path"
    }

    fn short_description(&self) -> &'static str {
        "Combine chained 'get' commands into cell paths"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/navigating_structured_data.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let config = ClusterConfig::min_consecutive(2);

        context
            .ast
            .pipelines
            .iter()
            .flat_map(|pipeline| {
                pipeline
                    .find_command_clusters("get", context, &config)
                    .into_iter()
                    .filter_map(|cluster| {
                        let start_idx = cluster.first_index()?;
                        let end_idx = cluster.last_index()?;

                        // Collect cell path members from each get call
                        let member_groups: Option<Vec<Vec<PathMember>>> = cluster
                            .indices
                            .iter()
                            .map(|&idx| extract_members_from_get_call(&pipeline.elements[idx]))
                            .collect();

                        let member_groups = member_groups?;
                        let combined_members: Vec<PathMember> =
                            member_groups.into_iter().flatten().collect();

                        // Create violation with labels
                        let num_gets = cluster.len();
                        let message = format!(
                            "Use combined cell path instead of {num_gets} chained 'get' commands"
                        );

                        let mut detection = Detection::from_global_span(message, cluster.span);
                        for &idx in &cluster.indices {
                            let span = pipeline.elements[idx].expr.span;
                            let label = get_command_label(idx, start_idx, end_idx);
                            detection = detection.with_extra_label(label, span);
                        }

                        let fix_data = FixData {
                            full_span: cluster.span,
                            combined_members,
                        };

                        Some((detection, fix_data))
                    })
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let combined_path = format_cell_path(&fix_data.combined_members);
        let replacement_text = format!("get {combined_path}");

        Some(Fix {
            explanation: "Combine into single cell path".into(),
            replacements: vec![Replacement::new(fix_data.full_span, replacement_text)],
        })
    }
}

pub static RULE: &dyn Rule = &MergeGetCellPath;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
