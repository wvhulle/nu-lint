use nu_protocol::ast::{Block, Expr};

use super::{AssignmentGroup, find_assignment_groups, format_path, make_detection};
use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const fn is_nested_non_env(group: &AssignmentGroup) -> bool {
    // Skip $env groups - can't reassign $env directly in nushell
    !group.all_flat && !group.is_env
}

fn detect_in_block<'a>(
    block: &'a Block,
    ctx: &'a LintContext<'a>,
) -> impl Iterator<Item = (Detection, AssignmentGroup)> + 'a {
    find_assignment_groups(block, ctx)
        .into_iter()
        .filter(is_nested_non_env)
        .map(|group| (make_detection(&group), group))
}

struct MergeNestedUpsert;

impl DetectFix for MergeNestedUpsert {
    type FixInput<'a> = AssignmentGroup;

    fn id(&self) -> &'static str {
        "merge_nested_upsert"
    }

    fn short_description(&self) -> &'static str {
        "Merge consecutive nested field assignments with upsert"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Multiple consecutive `$var.path.to.field = value` assignments can be merged into \
             `$var = ($var | upsert path.to.field1 value1 | upsert path.to.field2 value2)`. This \
             makes the intent clearer when setting multiple nested fields at once.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/upsert.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        detect_in_block(context.ast, context)
            .chain(context.detect_with_fix_data(|expr, ctx| match &expr.expr {
                Expr::Closure(block_id) | Expr::Block(block_id) => {
                    detect_in_block(ctx.working_set.get_block(*block_id), ctx).collect()
                }
                _ => Vec::new(),
            }))
            .collect()
    }

    fn fix(&self, context: &LintContext, group: &Self::FixInput<'_>) -> Option<Fix> {
        let upserts = group
            .assignments
            .iter()
            .map(|a| {
                format!(
                    "upsert {} {}",
                    format_path(&a.path),
                    context.span_text(a.value_span)
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");

        let var = &group.root_var_name;
        Some(Fix {
            explanation: format!(
                "Merge {} ${var} assignments with chained upsert",
                group.assignments.len()
            )
            .into(),
            replacements: vec![Replacement::new(
                group.combined_span,
                format!("${var} = (${var} | {upserts})"),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &MergeNestedUpsert;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
