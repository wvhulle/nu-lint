use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'find' for simple text search or 'lines | where' to leverage structured \
                    filtering. Nushell commands integrate with pipelines and are case-insensitive \
                    by default.";

#[derive(Default)]
struct RipgrepFlags {
    case_insensitive: bool,
    invert_match: bool,
    line_number: bool,
    count: bool,
    files_with_matches: bool,
    fixed_strings: bool,
}

#[derive(Default)]
struct RipgrepOptions {
    pattern: Option<String>,
    files: Vec<String>,
    flags: RipgrepFlags,
}

impl RipgrepOptions {
    #[allow(
        clippy::excessive_nesting,
        reason = "Single loop expansion for clarity of logging each file"
    )]
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter();
        log::debug!("rg.parse: starting");

        while let Some(arg) = iter.next() {
            match arg {
                "-i" | "--ignore-case" => opts.flags.case_insensitive = true,
                "-v" | "--invert-match" => opts.flags.invert_match = true,
                "-n" | "--line-number" => opts.flags.line_number = true,
                "-c" | "--count" => opts.flags.count = true,
                "-l" | "--files-with-matches" => opts.flags.files_with_matches = true,
                "-F" | "--fixed-strings" => opts.flags.fixed_strings = true,
                "-e" | "--regexp" => {
                    if let Some(pattern) = iter.next() {
                        Self::set_pattern(&mut opts, pattern);
                        log::debug!("rg.parse: pattern via -e => {pattern}");
                    }
                }
                "--" => {
                    {
                        for rest in iter {
                            opts.files.push(rest.to_string());
                            log::debug!("rg.parse: file after -- => {rest}");
                        }
                    }
                    break;
                }
                s if s.starts_with('-') && s.len() > 2 && !s.starts_with("--") => {
                    Self::parse_combined_flags(&mut opts, s);
                    log::debug!("rg.parse: combined flags => {s}");
                }
                s if s.starts_with("--") => {
                    // ignore other long flags for now
                }
                other => Self::set_pattern(&mut opts, other),
            }
        }
        log::debug!(
            "rg.parse: done pattern={:?} files={:?} flags=ci={} inv={} n={} c={} l={} F={}",
            opts.pattern,
            opts.files,
            opts.flags.case_insensitive,
            opts.flags.invert_match,
            opts.flags.line_number,
            opts.flags.count,
            opts.flags.files_with_matches,
            opts.flags.fixed_strings
        );

        opts
    }

    fn parse_combined_flags(opts: &mut Self, flags: &str) {
        for c in flags.chars().skip(1) {
            match c {
                'i' => opts.flags.case_insensitive = true,
                'v' => opts.flags.invert_match = true,
                'n' => opts.flags.line_number = true,
                'c' => opts.flags.count = true,
                'l' => opts.flags.files_with_matches = true,
                'F' => opts.flags.fixed_strings = true,
                _ => {}
            }
        }
    }

    #[allow(
        clippy::excessive_nesting,
        reason = "Pattern/file heuristic splitting needs localized nesting; readability \
                  preferable over splitting into many helpers"
    )]
    fn set_pattern(opts: &mut Self, value: &str) {
        if opts.pattern.is_some() {
            opts.files.push(value.to_string());
            log::debug!("rg.set_pattern: already had pattern; pushing file={value}");
            return;
        }

        if !value.contains(' ') {
            opts.pattern = Some(value.to_string());
            log::debug!("rg.set_pattern: simple pattern={value}");
            return;
        }

        // Attempt to separate a quoted pattern from trailing file names.
        let mut pattern_slice = value;
        let mut files_slice = "";
        if let Some(q) = value.chars().next().filter(|c| *c == '"' || *c == '\'')
            && let Some(close_idx) = value[q.len_utf8()..].find(q)
        {
            let closing_pos = close_idx + 2 * q.len_utf8();
            if closing_pos < value.len() {
                pattern_slice = &value[..closing_pos];
                files_slice = value[closing_pos..].trim();
                log::debug!(
                    "rg.set_pattern: quoted pattern_slice={pattern_slice} \
                     files_slice={files_slice}"
                );
            }
        }

        if files_slice.is_empty() {
            // Fallback: split on whitespace; first token pattern, rest files.
            let mut it = value.split_whitespace();
            if let Some(first) = it.next() {
                pattern_slice = first;
            }
            let remaining: Vec<String> = it.map(str::to_string).collect();
            opts.pattern = Some(pattern_slice.to_string());
            opts.files.extend(remaining);
            log::debug!(
                "rg.set_pattern: fallback pattern_slice={pattern_slice} files={:?}",
                opts.files
            );
        } else {
            opts.pattern = Some(pattern_slice.to_string());
            opts.files
                .extend(files_slice.split_whitespace().map(str::to_string));
            log::debug!(
                "rg.set_pattern: split pattern_slice={pattern_slice} files={:?}",
                opts.files
            );
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let raw = self.pattern.as_deref().unwrap_or("pattern");
        // Remove leading ^ anchor if present and surrounding quotes, then unescape
        // embedded quotes
        let inner = raw.strip_prefix('^').unwrap_or(raw);
        // Remove surrounding escaped quotes like \"pattern\"
        let de_escaped = inner
            .trim()
            .trim_start_matches("\\\"")
            .trim_end_matches("\\\"");
        // Then remove plain quotes
        let unquoted = de_escaped.trim().trim_matches('"').trim_matches('\'');
        log::debug!(
            "rg.to_nushell: raw={raw} unquoted={unquoted} files={:?}",
            self.files
        );
        if self.should_use_find() {
            self.build_find_replacement(unquoted)
        } else {
            self.build_where_replacement(unquoted)
        }
    }

    const fn should_use_find(&self) -> bool {
        !self.flags.invert_match
            && !self.flags.line_number
            && !self.flags.count
            && !self.flags.files_with_matches
            && self.files.is_empty()
    }

    fn build_find_replacement(&self, pattern: &str) -> (String, String) {
        // Tests expect escaped quotes around pattern: find \"pattern\"
        let escaped_inner = pattern.replace('"', "\\\"");
        let replacement = format!("find \\\"{escaped_inner}\\\"");
        let mut explanation = vec![format!("Use 'find \"{pattern}\"' for quick searches.")];

        if self.flags.case_insensitive {
            explanation.push(
                "The -i flag is redundant; Nu's 'find' is case-insensitive by default.".into(),
            );
        } else {
            explanation.push(
                "Nu's 'find' is case-insensitive by default and works on structured data.".into(),
            );
        }

        (replacement, explanation.join(" "))
    }

    fn build_where_replacement(&self, pattern: &str) -> (String, String) {
        let prefix = if self.files.is_empty() {
            "lines".to_string()
        } else {
            format!("open {} | lines", self.files.join(" "))
        };

        let (filter, examples) = self.build_where_filter(pattern);
        let mut pipeline_parts = vec![prefix];

        if self.flags.line_number {
            pipeline_parts.push("enumerate".into());
        }

        pipeline_parts.push(filter);
        let replacement = pipeline_parts.join(" | ");
        let description = self.build_where_description(pattern, &examples);

        (replacement, description)
    }

    fn build_where_filter(&self, pattern: &str) -> (String, Vec<String>) {
        let value_expr = if self.flags.line_number {
            "$it.item"
        } else {
            "$it"
        };
        let mut explanation = Vec::new();
        let quoted = format!("\\\"{}\\\"", pattern.replace('"', "\\\""));

        let predicate = if self.flags.fixed_strings {
            explanation.push("literal: use 'str contains' for fixed matches".into());
            format!("{value_expr} | str contains {quoted}")
        } else if self.flags.invert_match {
            explanation.push("invert matches with '!~'".into());
            format!("{value_expr} !~ {quoted}")
        } else {
            explanation.push("regex: '=~' for pattern matches".into());
            format!("{value_expr} =~ {quoted}")
        };

        let mut parts = vec![format!("where {predicate}")];

        if self.flags.count {
            explanation.push("count: pipe to 'length'".into());
            parts.push("length".into());
        }

        (parts.join(" | "), explanation)
    }

    fn build_where_description(&self, pattern: &str, examples: &[String]) -> String {
        let mut description = vec![format!(
            "Use 'lines | where' to filter for '{pattern}' with Nushell's structured pipelines."
        )];

        if !examples.is_empty() {
            description.push(format!("Pipeline stages: {}", examples.join(", ")));
        }

        if self.flags.case_insensitive {
            description.push(
                "case-insensitive matching can be done by downcasing input or using 'find'.".into(),
            );
        }

        if self.flags.files_with_matches {
            description.push(
                "For '-l', collect filenames after filtering with 'wrap file' or 'uniq'.".into(),
            );
        }

        description.push(
            "Nushell commands keep data structured, which simplifies post-processing.".into(),
        );

        description.join(" ")
    }
}

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let options = RipgrepOptions::parse(external_args_slices(args, context));
    let (replacement, explanation) = options.to_nushell();
    log::debug!(
        "rg.build_fix: replacement={replacement} explanation_len={}",
        explanation.len()
    );

    Fix {
        explanation: explanation.into(),
        replacements: vec![Replacement {
            span: expr_span.into(),
            replacement_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "rg", NOTE, Some(build_fix))
}

pub const fn rule() -> Rule {
    Rule::new(
        "use_builtin_rg",
        "Use Nu's 'find' or 'where' instead of 'rg'",
        check,
        LintLevel::Hint,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/find.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
