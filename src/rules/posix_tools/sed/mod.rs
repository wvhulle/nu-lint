use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'str replace' for text substitution";

struct UseBuiltinSed;

impl DetectFix for UseBuiltinSed {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "sed_to_str_replace"
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

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let validator = |_cmd: &str, args: &[&str]| {
            // Only detect simple substitution patterns that str replace can handle
            let has_unsupported = args.iter().any(|arg| {
                // External script file
                *arg == "-f" || (arg.starts_with("-f") && arg.len() > 2) ||
                // Non-substitution sed commands (looking for /d, /p, etc. patterns)
                (arg.contains('/') && (
                    arg.ends_with("/d") || arg.ends_with("/p") || 
                    arg.ends_with("/a") || arg.ends_with("/i") || 
                    arg.ends_with("/c")
                )) ||
                // Multiple commands separated by semicolon
                (arg.contains(';') && arg.contains('/')) ||
                // Print mode (changes behavior significantly)
                *arg == "-n" || *arg == "--quiet" || *arg == "--silent"
            });

            // Check if there's at least one substitution pattern
            let has_substitution = args.iter().any(|arg| {
                let trimmed = arg.trim_matches('"').trim_matches('\'');
                trimmed.starts_with("s/") || trimmed.contains(" s/")
            });

            if has_unsupported || !has_substitution {
                None
            } else {
                Some(NOTE)
            }
        };
        let mut violations = context.detect_external_with_validation("sed", validator);
        violations.extend(context.detect_external_with_validation("gsed", validator));
        violations
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = parse_sed_args(fix_data.arg_strings(_context));

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
    let mut file = None;
    let mut in_place = false;
    let mut regex_mode = false;
    let mut expect_expression = false;

    let args: Vec<&str> = args.into_iter().collect();
    let mut i = 0;

    while i < args.len() {
        let arg = args[i];

        if expect_expression {
            pattern = Some(arg);
            expect_expression = false;
            i += 1;
            continue;
        }

        match arg {
            "-i" | "--in-place" => in_place = true,
            "-e" | "--expression" => {
                expect_expression = true;
                if i + 1 < args.len() {
                    i += 1;
                    pattern = Some(args[i]);
                    expect_expression = false;
                }
            }
            "-E" | "-r" | "--regexp-extended" => regex_mode = true,
            s if s.starts_with('-') && s.len() > 1 && !s.starts_with("--") => {
                let flags = parse_combined_flags(s);
                in_place = in_place || flags.in_place;
                regex_mode = regex_mode || flags.regex_mode;
                if flags.expect_expression && i + 1 < args.len() {
                    i += 1;
                    pattern = Some(args[i]);
                }
            }
            s if pattern.is_none() => pattern = Some(s),
            s => file = Some(s),
        }
        i += 1;
    }

    let pattern_str = pattern.unwrap_or("s/pattern/replacement/");
    let (find, replace, global) = parse_substitution(pattern_str);

    build_str_replace_command(find, replace, file, in_place, regex_mode, global)
}

struct CombinedFlags {
    in_place: bool,
    regex_mode: bool,
    expect_expression: bool,
}

fn parse_combined_flags(arg: &str) -> CombinedFlags {
    let mut flags = CombinedFlags {
        in_place: false,
        regex_mode: false,
        expect_expression: false,
    };

    for ch in arg.chars().skip(1) {
        match ch {
            'i' => flags.in_place = true,
            'e' => flags.expect_expression = true,
            'E' | 'r' => flags.regex_mode = true,
            _ => {}
        }
    }

    flags
}

fn parse_substitution(pattern: &str) -> (&str, &str, bool) {
    let clean = pattern.trim_matches('"').trim_matches('\'');

    if !clean.starts_with('s') {
        return ("pattern", "replacement", false);
    }

    // Find delimiter (typically '/' but could be others)
    let delimiter = clean.chars().nth(1).unwrap_or('/');

    // Split by delimiter, handling escaped delimiters
    let parts: Vec<&str> = clean[2..].split(delimiter).collect();

    if parts.len() < 2 {
        return ("pattern", "replacement", false);
    }

    let find = parts[0];
    let replace = parts.get(1).copied().unwrap_or("");
    let flags = parts.get(2).copied().unwrap_or("");
    let global = flags.contains('g');

    (find, replace, global)
}

fn build_str_replace_command(
    find: &str,
    replace: &str,
    file: Option<&str>,
    in_place: bool,
    regex_mode: bool,
    global: bool,
) -> String {
    let mut flags = String::new();

    if global {
        flags.push_str(" --all");
    }
    if regex_mode {
        flags.push_str(" --regex");
    }

    // Escape single quotes in find and replace patterns
    let find_escaped = find.replace('\'', "''");
    let replace_escaped = replace.replace('\'', "''");

    match (file, in_place) {
        (Some(f), true) => {
            format!(
                "open {f} | str replace{flags} '{find_escaped}' '{replace_escaped}' | save -f {f}"
            )
        }
        (Some(f), false) => {
            format!("open {f} | str replace{flags} '{find_escaped}' '{replace_escaped}'")
        }
        _ => format!("str replace{flags} '{find_escaped}' '{replace_escaped}'"),
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
