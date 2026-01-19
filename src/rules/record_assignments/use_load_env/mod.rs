use nu_protocol::ast::{Block, Expr};

use super::{AssignmentGroup, find_assignment_groups, make_detection};
use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const fn is_flat_env(group: &AssignmentGroup) -> bool {
    group.is_env && group.all_flat
}

fn detect_in_block<'a>(
    block: &'a Block,
    ctx: &'a LintContext<'a>,
) -> impl Iterator<Item = (Detection, AssignmentGroup)> + 'a {
    find_assignment_groups(block, ctx)
        .into_iter()
        .filter(is_flat_env)
        .map(|group| (make_detection(&group), group))
}

struct UseLoadEnv;

impl DetectFix for UseLoadEnv {
    type FixInput<'a> = AssignmentGroup;

    fn id(&self) -> &'static str {
        "use_load_env"
    }

    fn short_description(&self) -> &'static str {
        "Use load-env for multiple $env assignments"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Multiple consecutive `$env.VAR = value` assignments can be replaced with a single \
             `load-env { VAR1: value1, VAR2: value2 }` call. This is more concise and idiomatic.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/load-env.html")
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
        let fields = group
            .assignments
            .iter()
            .map(|a| {
                format!(
                    "{}: {}",
                    a.path[0].to_record_key(),
                    context.span_text(a.value_span)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        Some(Fix {
            explanation: format!(
                "Use load-env for {} $env assignments",
                group.assignments.len()
            )
            .into(),
            replacements: vec![Replacement::new(
                group.combined_span,
                format!("load-env {{ {fields} }}"),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &UseLoadEnv;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
