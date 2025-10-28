use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};


fn check_for_inefficient_json_flow(
    context: &LintContext,
    pipeline: &nu_protocol::ast::Pipeline,
) -> Option<RuleViolation> {
    let elements = &pipeline.elements;

    // Look for patterns like: data | to json | ^jq 'filter'
    for window in elements.windows(2) {
        let first = &window[0];
        let second = &window[1];

        // Check if first element converts to JSON string
        if let Expr::Call(call) = &first.expr.expr {
            let first_name = context.working_set.get_decl(call.decl_id).name();

            if first_name == "to json" {
                // Check if second element uses jq
                if let Expr::ExternalCall(head, _) = &second.expr.expr {
                    let cmd_text = &context.source[head.span.start..head.span.end];
                    if cmd_text == "jq" {
                        return Some(
                            RuleViolation::new_static(
                                "prefer_structured_data_flow",
                                "Converting to JSON string just to use jq - consider keeping data \
                                 structured and using Nushell operations",
                                nu_protocol::Span::new(first.expr.span.start, second.expr.span.end),
                            )
                            .with_suggestion_static(
                                "Keep data in structured format and use Nushell commands like \
                                 'where', 'each', 'get' instead of converting to JSON and using jq",
                            ),
                        );
                    }
                }
            }
        }
    }

    None
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    // Check the main AST block directly
    for pipeline in &context.ast.pipelines {
        if let Some(violation) = check_for_inefficient_json_flow(context, pipeline) {
            violations.push(violation);
        }
    }

    // Also traverse the AST to check pipelines in nested blocks
    violations.extend(context.collect_rule_violations(|expr, ctx| {
        match &expr.expr {
            Expr::Block(block_id) => {
                let block = ctx.working_set.get_block(*block_id);
                let mut nested_violations = Vec::new();

                for pipeline in &block.pipelines {
                    if let Some(violation) = check_for_inefficient_json_flow(ctx, pipeline) {
                        nested_violations.push(violation);
                    }
                }
                nested_violations
            }
            _ => vec![],
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
