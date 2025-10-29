use nu_protocol::{Span, ast::Expr};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn has_error_handling_in_pipeline(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> bool {
    use nu_protocol::ast::Traverse;

    let mut found_handling = Vec::new();

    for element in &pipeline.elements {
        element.expr.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::Call(call) = &expr.expr {
                    let decl_name = context.working_set.get_decl(call.decl_id).name();
                    if decl_name == "try" || decl_name == "complete" {
                        return vec![true];
                    } else if decl_name == "do" {
                        for arg in &call.arguments {
                            if let nu_protocol::ast::Argument::Named(named) = arg
                                && named.0.item == "ignore"
                            {
                                return vec![true];
                            }
                        }
                    }
                }
                vec![]
            },
            &mut found_handling,
        );
    }

    !found_handling.is_empty()
}

fn has_external_command_in_pipeline(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> Option<Span> {
    use nu_protocol::ast::Traverse;

    for element in &pipeline.elements {
        let mut found_external = Vec::new();

        element.expr.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::ExternalCall(_head, _args) = &expr.expr {
                    return vec![expr.span];
                }
                vec![]
            },
            &mut found_external,
        );

        if !found_external.is_empty() {
            return found_external.first().copied();
        }
    }

    None
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context
        .ast
        .pipelines
        .iter()
        .filter(|pipeline| pipeline.elements.len() >= 2)
        .filter_map(|pipeline| {
            let has_error_handling = has_error_handling_in_pipeline(pipeline, context);
            let external_span = has_external_command_in_pipeline(pipeline, context)?;

            (!has_error_handling).then(|| {
                RuleViolation::new_static(
                    "require_external_command_error_check",
                    "External command in pipeline without error checking - exit code ignored",
                    external_span,
                )
                .with_suggestion_static(
                    "Add error checking: use 'complete' to capture exit codes, then check with \
                     'if $in.exit_code != 0 { ... }'",
                )
            })
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "require_external_command_error_check",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Require explicit error checking when piping external command output",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
