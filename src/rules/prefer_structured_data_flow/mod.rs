use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn check_pipeline_for_violations(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> Option<RuleViolation> {
    // Look for patterns like: data | to json | ^jq 'filter' using sliding window
    pipeline.elements.windows(2).find_map(|window| {
        let (first, second) = (&window[0], &window[1]);

        // Check if first element converts to JSON string and second uses jq
        if let (Expr::Call(call), Expr::ExternalCall(head, _)) =
            (&first.expr.expr, &second.expr.expr)
        {
            let first_name = context.working_set.get_decl(call.decl_id).name();
            let cmd_text = &context.source[head.span.start..head.span.end];

            if first_name == "to json" && cmd_text == "jq" {
                Some(
                    RuleViolation::new_static(
                        "prefer_structured_data_flow",
                        "Converting to JSON string just to use jq - consider keeping data \
                         structured and using Nushell operations",
                        nu_protocol::Span::new(first.expr.span.start, second.expr.span.end),
                    )
                    .with_suggestion_static(
                        "Keep data in structured format and use Nushell commands like 'where', \
                         'each', 'get' instead of converting to JSON and using jq",
                    ),
                )
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    // Check the main AST block directly
    for pipeline in &context.ast.pipelines {
        if let Some(violation) = check_pipeline_for_violations(pipeline, context) {
            violations.push(violation);
        }
    }

    // Collect violations from all nested blocks (including function bodies)
    violations.extend(context.collect_rule_violations(|expr, ctx| {
        match &expr.expr {
            Expr::Block(block_id) => {
                let block = ctx.working_set.get_block(*block_id);
                let mut nested_violations = Vec::new();

                // Check all pipelines in this block
                for pipeline in &block.pipelines {
                    if let Some(violation) = check_pipeline_for_violations(pipeline, ctx) {
                        nested_violations.push(violation);
                    }
                }
                nested_violations
            }
            Expr::Closure(block_id) => {
                let block = ctx.working_set.get_block(*block_id);
                let mut nested_violations = Vec::new();

                // Check all pipelines in this closure block (function body)
                for pipeline in &block.pipelines {
                    if let Some(violation) = check_pipeline_for_violations(pipeline, ctx) {
                        nested_violations.push(violation);
                    }
                }
                nested_violations
            }
            _ => {
                vec![]
            }
        }
    }));

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_structured_data_flow",
        RuleCategory::Idioms,
        Severity::Info,
        "Prefer keeping data in structured format throughout the pipeline instead of converting \
         to JSON strings for jq processing",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
