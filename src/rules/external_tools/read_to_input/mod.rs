use std::borrow::Cow;

use crate::{
    LintLevel,
    ast::{
        external::{CliSpec, ExternalInvocation, Flag, FlagValue, literal_content, value_text},
        string::quote_nu_string,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const PROMPT: Flag = Flag::short_switch('p').with_value();
const SILENT: Flag = Flag::short_switch('s');
const RAW: Flag = Flag::short_switch('r');

static SPEC: CliSpec = CliSpec {
    flags: &[PROMPT, SILENT, RAW],
    numeric_shorthand: None,
};

const NOTE: &str = "'read' is a shell builtin: an external 'read' cannot assign a variable in the \
                    script and usually does not exist as a binary. Use 'input' and bind its \
                    result with 'let'.";

fn is_variable_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn prompt_text<'a>(prompt: FlagValue<'a>, context: &'a LintContext) -> Cow<'a, str> {
    match prompt {
        FlagValue::Inline(text) => Cow::Owned(quote_nu_string(text)),
        FlagValue::Argument(expr) => value_text(context, expr),
    }
}

fn input_binding(invocation: &ExternalInvocation, context: &LintContext) -> Option<String> {
    if invocation.previous.is_some() || invocation.next.is_some() {
        return None;
    }
    let parsed = invocation.parse(&SPEC)?;
    let [variable] = parsed.operands.as_slice() else {
        return None;
    };
    let name = literal_content(variable).filter(|name| is_variable_name(name))?;
    let suppress = if parsed.has(SILENT) {
        " --suppress-output"
    } else {
        ""
    };
    let prompt = parsed.value(PROMPT).map_or(String::new(), |prompt| {
        format!(" {}", prompt_text(prompt, context))
    });
    Some(format!("let {name} = input{suppress}{prompt}"))
}

struct ReadToInput;

impl DetectFix for ReadToInput {
    type FixInput<'a> = Option<Replacement>;

    fn id(&self) -> &'static str {
        "read_to_input"
    }

    fn short_description(&self) -> &'static str {
        "External `read` replaceable with `input`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/input.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["read"])
            .iter()
            .map(|invocation| {
                let replacement = input_binding(invocation, context)
                    .map(|binding| Replacement::new(invocation.span, binding));
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'read' cannot set a variable");
                (detection, replacement)
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = replacement.as_ref()?;
        Some(Fix {
            explanation: format!("Read input with '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
        })
    }
}

pub static RULE: &dyn Rule = &ReadToInput;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
