use nu_protocol::ast::{Call, Expr};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

/// Check if a call is to the 'exit' command
fn is_exit_call(call: &Call, ctx: &LintContext) -> bool {
    call.get_call_name(ctx) == "exit"
}

struct ExitOnlyInMain;

impl DetectFix for ExitOnlyInMain {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "exit_only_in_main"
    }

    fn short_description(&self) -> &'static str {
        "Avoid using 'exit' in functions other than 'main'"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/exit.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // First, collect all function definitions
        let functions = context.custom_commands();

        // Then, find all exit calls and check if they're in non-main functions
        let violations = context.detect_single(|expr, ctx| {
            if let Expr::Call(call) = &expr.expr {
                if !is_exit_call(call, ctx) {
                    return None;
                }

                // Check if this exit is inside a function
                let function_def = call.head.find_containing_function(&functions, ctx)?;

                // Allow exit in main function
                if function_def.is_main() {
                    return None;
                }

                return Some(
                    Detection::from_global_span(
                        format!(
                            "Function '{}' uses 'exit' which terminates the entire script",
                            function_def.name
                        ),
                        call.head,
                    )
                    .with_primary_label("exit call")
                    .with_extra_label(format!("inside '{}'", function_def.name), expr.span),
                );
            }
            None
        });

        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &ExitOnlyInMain;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
