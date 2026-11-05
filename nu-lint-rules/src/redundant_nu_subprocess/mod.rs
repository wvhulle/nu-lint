use nu_lint_analysis::{
    ast::external::literal_content,
    config::LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, DiagnosticTag},
};
use nu_protocol::ast::{Expr, Expression, ExternalArgument};

struct AvoidNuSubprocess;

impl DetectFix for AvoidNuSubprocess {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "redundant_nu_subprocess"
    }

    fn short_description(&self) -> &'static str {
        "Redundant `nu -c` subprocess call"
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::Unnecessary]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, _ctx| Self::check_expression(expr)))
    }
}

impl AvoidNuSubprocess {
    fn check_expression(expr: &Expression) -> Vec<Detection> {
        let Expr::ExternalCall(head, args) = &expr.expr else {
            return vec![];
        };

        if !Self::is_nu_with_c_flag(head, args) {
            return vec![];
        }

        vec![
            Detection::from_global_span(
                "Avoid spawning `nu -c` subprocess from within a Nu script",
                expr.span,
            )
            .with_primary_label("subprocess spawned here"),
        ]
    }

    fn is_nu_with_c_flag(command_head: &Expression, args: &[ExternalArgument]) -> bool {
        if literal_content(command_head) != Some("nu") {
            return false;
        }

        args.iter().any(|arg| {
            let ExternalArgument::Regular(expr) = arg else {
                return false;
            };
            matches!(
                &expr.expr,
                Expr::String(argument) | Expr::RawString(argument) | Expr::GlobPattern(argument, _)
                    if matches!(argument.as_str(), "-c" | "--commands")
            )
        })
    }
}

pub static RULE: &dyn Rule = &AvoidNuSubprocess;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
