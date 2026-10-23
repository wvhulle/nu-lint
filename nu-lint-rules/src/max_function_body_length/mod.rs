use crate::{
    LintLevel,
    ast::declaration,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
fn check(context: &LintContext) -> Vec<Detection> {
    let max = context.config.max_function_body_statements;
    context
        .custom_commands()
        .iter()
        .filter_map(|def| function_violation(context, def, max))
        .collect()
}

fn function_violation(
    context: &LintContext<'_>,
    def: &declaration::CustomCommandDef,
    max: usize,
) -> Option<Detection> {
    let block = context.working_set.get_block(def.body);
    let pipeline_count = block.pipelines.len();
    (pipeline_count > max).then(|| {
        let message = format!(
            "Function `{}` has {pipeline_count} statements, which exceeds the maximum of {max} \
             statements",
            def.name
        );
        let detection = Detection::from_file_span(message, def.declaration_span(context))
            .with_primary_label(format!("{pipeline_count} statements"));
        if let Some(span) = block.span {
            detection.with_extra_label("function body", span)
        } else {
            detection
        }
    })
}
struct TooManyLines;

impl DetectFix for TooManyLines {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "max_function_body_length"
    }

    fn short_description(&self) -> &'static str {
        "Function bodies should be short to maintain readability"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Consider refactoring into smaller, more focused functions with clear responsibilities",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(check(context))
    }
}

pub static RULE: &dyn Rule = &TooManyLines;
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
