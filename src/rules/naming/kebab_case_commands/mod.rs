use nu_protocol::ast::Expr;

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn to_kebab_case_preserving_spaces(name: &str) -> String {
    name.split(' ')
        .map(heck::ToKebabCase::to_kebab_case)
        .collect::<Vec<_>>()
        .join(" ")
}

fn check(context: &LintContext) -> Vec<Detection> {
    context.detect(|expr, ctx| {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        let Some(func_def) = call.custom_command_def(ctx) else {
            return vec![];
        };

        let cmd_name = &func_def.name;
        let name_span = func_def.name_span;

        let kebab_case_name = to_kebab_case_preserving_spaces(cmd_name);

        if cmd_name.as_str() == kebab_case_name {
            return vec![];
        }

        vec![
            Detection::from_global_span(
                format!("Command '{cmd_name}' should follow naming convention"),
                name_span,
            )
            .with_primary_label("non-kebab-case name"),
        ]
    })
}

struct KebabCaseCommands;

impl DetectFix for KebabCaseCommands {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "kebab_case_commands"
    }

    fn short_description(&self) -> &'static str {
        "Custom commands should use kebab-case naming convention"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#commands")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(check(context))
    }
}

pub static RULE: &dyn Rule = &KebabCaseCommands;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
