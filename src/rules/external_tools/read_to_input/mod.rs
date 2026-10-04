use nu_protocol::Span;

use crate::{
    LintLevel,
    ast::external::{CliSpec, ExternalInvocation, Flag, FlagValue, literal_content},
    context::LintContext,
    dsl::arguments::is_shell_variable_name,
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

pub struct InputBinding<'a> {
    span: Span,
    variable: &'a str,
    silent: bool,
    prompt: Option<FlagValue<'a>>,
}

impl<'a> InputBinding<'a> {
    fn from_invocation(invocation: &ExternalInvocation<'a>) -> Option<Self> {
        if invocation.previous.is_some() || invocation.next.is_some() {
            return None;
        }
        let parsed = invocation.parse(&SPEC)?;
        let [variable] = parsed.operands.as_slice() else {
            return None;
        };
        let variable = literal_content(variable)?;
        if !is_shell_variable_name(variable) {
            return None;
        }
        Some(Self {
            span: invocation.span,
            variable,
            silent: parsed.has(SILENT),
            prompt: parsed.value(PROMPT),
        })
    }

    fn to_nu(&self, context: &LintContext) -> String {
        let suppress = if self.silent {
            " --suppress-output"
        } else {
            ""
        };
        let prompt = self.prompt.map_or(String::new(), |prompt| {
            format!(" {}", prompt.nu_text(context))
        });
        format!("let {} = input{suppress}{prompt}", self.variable)
    }
}

struct ReadToInput;

impl DetectFix for ReadToInput {
    type FixInput<'a> = Option<InputBinding<'a>>;

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
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'read' cannot set a variable");
                (detection, InputBinding::from_invocation(invocation))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, binding: &Self::FixInput<'_>) -> Option<Fix> {
        let binding = binding.as_ref()?;
        let replacement = binding.to_nu(context);
        Some(Fix {
            explanation: format!("Read input with '{replacement}'").into(),
            replacements: vec![Replacement::new(binding.span, replacement)],
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
