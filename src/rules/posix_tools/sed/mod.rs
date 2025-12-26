use crate::{
    LintLevel,
    context::LintContext,
    external_commands::{ExternalCmdFixData, detect_external_commands, external_args_slices},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'str replace' for text substitution";

struct UseBuiltinSed;

impl DetectFix for UseBuiltinSed {
    type FixInput = ExternalCmdFixData;

    fn id(&self) -> &'static str {
        "use_builtin_sed"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'str replace' instead of 'sed'"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/str_replace.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        let mut violations = detect_external_commands(context, "sed", NOTE);
        violations.extend(detect_external_commands(context, "gsed", NOTE));
        violations
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let replacement = parse_sed_args(external_args_slices(&fix_data.args, context));

        Some(Fix {
            explanation: "Replace with str replace".into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinSed;

fn parse_sed_args<'a>(args: impl IntoIterator<Item = &'a str>) -> String {
    let mut pattern = None;
    let mut global = false;
    let mut file = None;
    let mut in_place = false;
    let mut regex_mode = false;
    let mut expect_expression = false;

    for arg in args {
        if expect_expression {
            pattern = Some(arg);
            global = arg.contains("/g");
            expect_expression = false;
            continue;
        }

        match arg {
            "-i" | "--in-place" => in_place = true,
            "-e" | "--expression" => expect_expression = true,
            "-E" | "-r" | "--regexp-extended" => regex_mode = true,
            "-n" | "--quiet" | "--silent" => {}
            s if s.starts_with('-') && !s.starts_with("--") => {
                (in_place, regex_mode, expect_expression) =
                    parse_combined_flags(s, in_place, regex_mode);
            }
            s if pattern.is_none() => {
                pattern = Some(s);
                global = s.contains("/g");
            }
            s => file = Some(s),
        }
    }

    let (find, replace) = parse_substitution(pattern.unwrap_or(""));
    let mut flags = String::new();
    if global {
        flags.push_str(" --all");
    }
    if regex_mode {
        flags.push_str(" --regex");
    }

    match (file, in_place) {
        (Some(f), true) => {
            format!("open {f} | str replace{flags} '{find}' '{replace}' | save -f {f}")
        }
        (Some(f), false) => format!("open {f} | str replace{flags} '{find}' '{replace}'"),
        _ => format!("str replace{flags} '{find}' '{replace}'"),
    }
}

fn parse_combined_flags(arg: &str, mut in_place: bool, mut regex_mode: bool) -> (bool, bool, bool) {
    let mut expect_expression = false;
    for ch in arg.chars().skip(1) {
        match ch {
            'i' => in_place = true,
            'e' => expect_expression = true,
            'E' | 'r' => regex_mode = true,
            _ => {}
        }
    }
    (in_place, regex_mode, expect_expression)
}

fn parse_substitution(pattern: &str) -> (&str, &str) {
    let clean = pattern.trim_matches('"').trim_matches('\'');

    if clean.starts_with('s') && clean.contains('/') {
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() >= 3 {
            return (parts[1], parts[2]);
        }
    }

    ("pattern", "replacement")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
