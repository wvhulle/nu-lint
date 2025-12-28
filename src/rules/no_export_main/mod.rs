use nu_protocol::ast::{Call, Expr};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, declaration::CustomCommandDef},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_export_main(call: &Call, context: &LintContext) -> Option<(Detection, CustomCommandDef)> {
    let func_def = call.custom_command_def(context)?;

    if !func_def.is_exported() {
        return None;
    }

    if !func_def.is_main() {
        return None;
    }

    let export_span = func_def.export_span?;

    let violation = Detection::from_global_span(
        format!(
            "Unnecessary 'export' keyword on main function '{}'",
            func_def.name
        ),
        export_span,
    )
    .with_primary_label("unnecessary export keyword")
    .with_extra_label("main function", func_def.name_span)
    .with_help(
        "Remove 'export' keyword - main functions are script entry points, not module exports"
            .to_string(),
    );

    Some((violation, func_def))
}

struct NoExportMain;

impl DetectFix for NoExportMain {
    type FixInput<'a> = CustomCommandDef;

    fn id(&self) -> &'static str {
        "no_export_main"
    }

    fn explanation(&self) -> &'static str {
        "Remove 'export' keyword from main functions - it's unnecessary for script entry points"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#subcommands")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            if let Expr::Call(call) = &expr.expr {
                check_export_main(call, ctx).into_iter().collect()
            } else {
                vec![]
            }
        })
    }

    fn fix(&self, _context: &LintContext, func_def: &Self::FixInput<'_>) -> Option<Fix> {
        let export_span = func_def.export_span?;
        Some(Fix::with_explanation(
            format!("Remove 'export' keyword from '{}'", func_def.name),
            vec![Replacement::new(export_span, "")],
        ))
    }
}

pub static RULE: &dyn Rule = &NoExportMain;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
