use nu_protocol::{
    Span,
    ast::{Argument, Block, Expr, Pipeline},
};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{block::BlockExt, call::CallExt, pipeline::PipelineExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct FixData {
    /// Span covering `lines | each { ... }`
    span: Span,
    /// The parse pattern extracted from inside the closure
    parse_pattern: String,
    /// Whether it uses --regex flag
    uses_regex: bool,
}

/// Extract parse pattern if closure body is exactly `$param | parse "pattern"`
fn extract_parse_from_closure(
    closure_block: &Block,
    closure_param_id: nu_protocol::VarId,
    ctx: &LintContext,
) -> Option<(String, bool)> {
    // Must be single pipeline
    if closure_block.pipelines.len() != 1 {
        log::debug!(
            "Closure has {} pipelines, expected 1",
            closure_block.pipelines.len()
        );
        return None;
    }
    let pipeline = &closure_block.pipelines[0];

    // Must be exactly 2 elements: $param | parse "..."
    if pipeline.elements.len() != 2 {
        log::debug!(
            "Closure pipeline has {} elements, expected 2",
            pipeline.elements.len()
        );
        return None;
    }

    // First element must be the closure parameter variable (can be Var or
    // FullCellPath with empty tail)
    let first = &pipeline.elements[0];
    let var_id = match &first.expr.expr {
        Expr::Var(id) => *id,
        Expr::FullCellPath(fcp) if fcp.tail.is_empty() => {
            if let Expr::Var(id) = &fcp.head.expr {
                *id
            } else {
                return None;
            }
        }
        _ => return None,
    };
    if var_id != closure_param_id {
        return None;
    }

    // Second element must be a parse call
    let second = &pipeline.elements[1];
    let Expr::Call(parse_call) = &second.expr.expr else {
        log::debug!("Second element is not a Call");
        return None;
    };
    if !parse_call.is_call_to_command("parse", ctx) {
        log::debug!("Second element is not parse command");
        return None;
    }

    // Extract the pattern argument
    let pattern_arg = parse_call.get_first_positional_arg()?;
    let pattern = match &pattern_arg.expr {
        Expr::String(s) | Expr::RawString(s) => s.clone(),
        _ => ctx.expr_text(pattern_arg).to_string(),
    };

    // Check if --regex flag is present
    let uses_regex = parse_call
        .arguments
        .iter()
        .any(|arg| matches!(arg, Argument::Named((name, _, _)) if name.item == "regex"));

    Some((pattern, uses_regex))
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    pipeline
        .find_command_pairs(
            context,
            |call, ctx| call.is_call_to_command("lines", ctx),
            |call, ctx| call.is_call_to_command("each", ctx),
        )
        .into_iter()
        .filter_map(|pair| {
            // Get the closure argument from each call
            let closure_arg = pair.second.arguments.iter().find_map(|arg| {
                if let Argument::Positional(expr) = arg
                    && let Expr::Closure(block_id) = &expr.expr
                {
                    return Some(*block_id);
                }
                None
            })?;

            let closure_block = context.working_set.get_block(closure_arg);

            // Get closure parameter
            let signature = &closure_block.signature;
            let param = signature.required_positional.first()?;
            let param_id = param.var_id?;

            // Check if closure body is just `$param | parse "..."`
            let (parse_pattern, uses_regex) =
                extract_parse_from_closure(closure_block, param_id, context)?;

            let violation = Detection::from_global_span(
                "Remove redundant 'each' wrapper around 'parse'",
                pair.span,
            )
            .with_primary_label("can be simplified")
            .with_extra_label("lines command", pair.first.span())
            .with_extra_label(
                "each closure just calls parse",
                Span::new(pair.second.head.start, pair.second.span().end),
            );

            Some((
                violation,
                FixData {
                    span: pair.span,
                    parse_pattern,
                    uses_regex,
                },
            ))
        })
        .collect()
}

struct LinesEachParseRule;

impl DetectFix for LinesEachParseRule {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "lines_each_to_parse"
    }

    fn short_description(&self) -> &'static str {
        "Remove redundant 'each' wrapper around 'parse'"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "The 'parse' command already operates on each line when used after 'lines'. Wrapping \
             it in 'each { |l| $l | parse ... }' is redundant. Use 'lines | parse ...' directly.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/parse.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = if fix_data.uses_regex {
            format!("lines | parse --regex \"{}\"", fix_data.parse_pattern)
        } else {
            format!("lines | parse \"{}\"", fix_data.parse_pattern)
        };

        Some(Fix {
            explanation: "simplify".into(),
            replacements: vec![Replacement::new(fix_data.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &LinesEachParseRule;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
