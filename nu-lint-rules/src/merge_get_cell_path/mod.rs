use nu_lint_analysis::{
    LintLevel,
    ast::{
        block::BlockExt,
        call::CallExt,
        pipeline::{ClusterConfig, PipelineExt},
        string::format_cell_path_members,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Span,
    ast::{Call, Expr, Expression, PathMember, Pipeline, PipelineElement},
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

const fn get_command_label(idx: usize, start_idx: usize, end_idx: usize) -> &'static str {
    if idx == start_idx {
        "First 'get' command"
    } else if idx == end_idx {
        "Last 'get' command"
    } else {
        "Intermediate 'get' command"
    }
}

fn extract_get_call(element: &PipelineElement) -> Option<&Call> {
    match &element.expr.expr {
        Expr::Call(get_call) => Some(get_call),
        _ => None,
    }
}

fn extract_members_from_get_call(
    get_call: &Call,
    context: &LintContext,
) -> Option<Vec<PathMember>> {
    let mut members = extract_cell_path_members(get_call.get_first_positional_arg()?)?;
    if get_call.is_get_optional(context) {
        members.iter_mut().for_each(PathMember::make_optional);
    }
    Some(members)
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    let config = ClusterConfig::min_consecutive(2);

    pipeline
        .find_command_clusters("get", context, &config)
        .into_iter()
        .filter_map(|cluster| {
            let start_idx = cluster.first_index()?;
            let end_idx = cluster.last_index()?;

            let get_calls: Vec<&Call> = cluster
                .indices
                .iter()
                .map(|&idx| extract_get_call(&pipeline.elements[idx]))
                .collect::<Option<_>>()?;

            let first_is_optional = get_calls.first()?.is_get_optional(context);
            if get_calls
                .iter()
                .any(|get_call| get_call.is_get_optional(context) != first_is_optional)
            {
                return None;
            }

            let member_groups: Vec<Vec<PathMember>> = get_calls
                .iter()
                .map(|get_call| extract_members_from_get_call(get_call, context))
                .collect::<Option<_>>()?;
            let combined_members: Vec<PathMember> = member_groups.into_iter().flatten().collect();

            let num_gets = cluster.len();
            let message =
                format!("Use combined cell path instead of {num_gets} chained 'get' commands");

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
        .collect()
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

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let combined_path = format_cell_path_members(&fix_data.combined_members);
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
