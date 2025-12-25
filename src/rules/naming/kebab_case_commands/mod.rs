use nu_protocol::ast::Expr;

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn strip_quotes(name: &str) -> &str {
    name.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(name)
}

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

        let decl_name = call.get_call_name(ctx);
        if !matches!(decl_name.as_str(), "def" | "export def") {
            return vec![];
        }

        let Some((raw_cmd_name, name_span)) = call.extract_declaration_name(ctx) else {
            return vec![];
        };

        let cmd_name = strip_quotes(&raw_cmd_name);
        let kebab_case_name = to_kebab_case_preserving_spaces(cmd_name);

        if cmd_name == kebab_case_name {
            return vec![];
        }

        vec![
            Detection::from_global_span(
                format!("Command '{cmd_name}' should follow naming convention"),
                name_span,
            )
            .with_primary_label("non-kebab-case name")
            .with_help(format!("Consider renaming to: {kebab_case_name}")),
        ]
    })
}

struct KebabCaseCommands;

impl DetectFix for KebabCaseCommands {
    type FixInput = ();

    fn id(&self) -> &'static str {
        "kebab_case_commands"
    }

    fn explanation(&self) -> &'static str {
        "Custom commands should use kebab-case naming convention"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#commands")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        Self::no_fix(check(context))
    }
}

pub static RULE: &dyn Rule = &KebabCaseCommands;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
