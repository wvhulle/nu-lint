use std::mem;

use nu_protocol::{
    Span,
    ast::{Block, Expr, Pipeline, Traverse},
};

use crate::{
    ast::{call::CallExt, string::StringFormat},
    config::LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const MIN_CONSECUTIVE_PRINTS: usize = 3;

/// Information extracted from a single `print` statement.
#[derive(Debug, Clone)]
struct PrintInfo {
    span: Span,
    string_type: StringFormat,
    to_stderr: bool,
}

/// Semantic fix data: stores the print group info needed for fix generation
pub struct FixData {
    prints: Vec<PrintInfo>,
    combined_span: Span,
}

impl PrintInfo {
    /// Attempts to extract print info from a pipeline.
    ///
    /// Returns `None` if the pipeline is not a simple `print "string"` or
    /// `print $"interpolation"` call.
    fn from_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Self> {
        let [element] = pipeline.elements.as_slice() else {
            return None;
        };

        let Expr::Call(call) = &element.expr.expr else {
            return None;
        };

        if !call.is_call_to_command("print", context) {
            return None;
        }

        let to_stderr = call.has_named_flag("stderr") || call.has_named_flag("e");
        let string_type = StringFormat::from_call_arg(call, context)?;

        Some(Self {
            span: element.expr.span,
            string_type,
            to_stderr,
        })
    }
}

fn find_print_groups(block: &Block, context: &LintContext) -> Vec<Vec<PrintInfo>> {
    let mut groups = Vec::new();
    let mut current_group: Vec<PrintInfo> = Vec::new();

    for pipeline in &block.pipelines {
        match PrintInfo::from_pipeline(pipeline, context) {
            Some(info) if can_extend_group(&current_group, &info) => {
                current_group.push(info);
            }
            Some(info) => {
                flush_group(&mut groups, &mut current_group);
                current_group.push(info);
            }
            None => {
                flush_group(&mut groups, &mut current_group);
            }
        }
    }

    flush_group(&mut groups, &mut current_group);
    groups
}

fn can_extend_group(group: &[PrintInfo], info: &PrintInfo) -> bool {
    group.first().is_none_or(|first| {
        first.to_stderr == info.to_stderr && first.string_type.is_compatible(&info.string_type)
    })
}

fn flush_group(groups: &mut Vec<Vec<PrintInfo>>, current: &mut Vec<PrintInfo>) {
    if current.len() >= MIN_CONSECUTIVE_PRINTS {
        groups.push(mem::take(current));
    } else {
        current.clear();
    }
}

/// Creates a violation for a group of consecutive print statements.
fn create_violation(prints: Vec<PrintInfo>) -> (Detection, FixData) {
    let combined_span = Span::new(
        prints.first().map_or(0, |p| p.span.start),
        prints.last().map_or(0, |p| p.span.end),
    );

    let violation = Detection::from_global_span(
        format!(
            "Found {} consecutive `print` statements with string literals that could be merged",
            prints.len()
        ),
        combined_span,
    )
    .with_primary_label("consecutive prints");

    let fix_data = FixData {
        prints,
        combined_span,
    };

    (violation, fix_data)
}

/// Recursively checks a block and all nested blocks for violations.
const fn extract_nested_block_id(expr: &Expr) -> Option<nu_protocol::BlockId> {
    match expr {
        Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => Some(*id),
        _ => None,
    }
}

fn detect_with_fix_data_from_nested_blocks(
    block: &Block,
    context: &LintContext,
    violations: &mut Vec<(Detection, FixData)>,
) {
    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            element.expr.flat_map(
                context.working_set,
                &|expr| {
                    extract_nested_block_id(&expr.expr)
                        .map(|id| detect_block(context.working_set.get_block(id), context))
                        .unwrap_or_default()
                },
                violations,
            );
        }
    }
}

fn detect_block(block: &Block, context: &LintContext) -> Vec<(Detection, FixData)> {
    let mut violations: Vec<_> = find_print_groups(block, context)
        .into_iter()
        .map(create_violation)
        .collect();

    detect_with_fix_data_from_nested_blocks(block, context, &mut violations);

    violations
}

fn escape_content_for_double_quotes(content: &str) -> String {
    content.replace('\\', r"\\").replace('"', r#"\""#)
}

fn process_string_for_merging(string_type: &StringFormat) -> String {
    match string_type {
        StringFormat::InterpolationDouble(_) | StringFormat::InterpolationSingle(_) => {
            string_type.content().to_string()
        }
        StringFormat::Double(_)
        | StringFormat::Single(_)
        | StringFormat::Raw(_)
        | StringFormat::BareWord(_) => escape_content_for_double_quotes(string_type.content()),
        StringFormat::Backtick(_) => string_type.content().to_string(),
    }
}

const fn determine_quote_style(first_string_type: Option<&StringFormat>) -> &'static str {
    match first_string_type {
        Some(StringFormat::InterpolationDouble(_)) => "$\"",
        Some(StringFormat::InterpolationSingle(_)) => "$'",
        _ => "\"",
    }
}

fn merge_print_contents(prints: &[PrintInfo]) -> String {
    prints
        .iter()
        .map(|p| process_string_for_merging(&p.string_type))
        .collect::<Vec<_>>()
        .join("\n")
}

fn has_stderr_flag(prints: &[PrintInfo]) -> bool {
    prints.first().is_some_and(|p| p.to_stderr)
}

fn build_replacement_text(merged_content: &str, quote_style: &str, stderr_flag: &str) -> String {
    match quote_style {
        "$\"" => format!("print{stderr_flag} $\"{merged_content}\""),
        "$'" => format!("print{stderr_flag} $'{merged_content}'"),
        _ => format!("print{stderr_flag} \"{merged_content}\""),
    }
}

struct MergeMultilinePrint;

impl DetectFix for MergeMultilinePrint {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "merge_multiline_print"
    }

    fn short_description(&self) -> &'static str {
        "Consecutive print statements can be merged into a single print with a multiline string"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/working_with_strings.html#string-formats-at-a-glance")
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Merge consecutive print statements into a single print with a multiline string for \
             cleaner code. Use `print \"line1\\nline2\\nline3\"` instead of multiple print calls.",
        )
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        detect_block(context.ast, context)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let prints = &fix_data.prints;
        let first_type = prints.first().map(|p| &p.string_type);

        let merged_content = merge_print_contents(prints);
        let quote_style = determine_quote_style(first_type);
        let stderr_flag = if has_stderr_flag(prints) { " -e" } else { "" };

        log::debug!("Merged content is: '{merged_content}'");

        let replacement_text = build_replacement_text(&merged_content, quote_style, stderr_flag);

        Some(Fix::with_explanation(
            format!(
                "Merge {} consecutive print statements into a single multiline print",
                prints.len()
            ),
            vec![Replacement::new(fix_data.combined_span, replacement_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &MergeMultilinePrint;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
