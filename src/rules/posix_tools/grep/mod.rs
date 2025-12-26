use crate::{
    LintLevel,
    context::LintContext,
    external_commands::ExternalCmdFixData,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'find' for simple text search (case-insensitive by default), 'where $it \
                    =~ pattern' for regex filtering, or 'lines | where' for line-based filtering \
                    with structured data operations.";

#[derive(Default)]
struct GrepFlags {
    case_insensitive: bool,
    invert_match: bool,
    line_number: bool,
    count: bool,
    files_with_matches: bool,
}

/// Parse grep command arguments to extract key options
#[derive(Default)]
struct GrepOptions {
    pattern: Option<String>,
    files: Vec<String>,
    flags: GrepFlags,
    extended_regex: bool,
    fixed_strings: bool,
    recursive: bool,
}

impl GrepOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            match arg {
                "-i" | "--ignore-case" => opts.flags.case_insensitive = true,
                "-v" | "--invert-match" => opts.flags.invert_match = true,
                "-n" | "--line-number" => opts.flags.line_number = true,
                "-c" | "--count" => opts.flags.count = true,
                "-l" | "--files-with-matches" => opts.flags.files_with_matches = true,
                "-E" | "--extended-regexp" => opts.extended_regex = true,
                "-F" | "--fixed-strings" => opts.fixed_strings = true,
                "-r" | "-R" | "--recursive" => opts.recursive = true,
                "-A" | "--after-context" | "-B" | "--before-context" | "-C" | "--context"
                | "-m" | "--max-count" => {
                    iter.next();
                }
                s if !s.starts_with('-') => Self::add_non_flag_arg(&mut opts, s),
                s if s.starts_with('-') && s.len() > 2 && !s.starts_with("--") => {
                    Self::parse_combined_flags(&mut opts, s);
                }
                _ => {}
            }
        }

        opts
    }

    fn parse_combined_flags(opts: &mut Self, flags: &str) {
        flags.chars().skip(1).for_each(|c| match c {
            'i' => opts.flags.case_insensitive = true,
            'v' => opts.flags.invert_match = true,
            'n' => opts.flags.line_number = true,
            'c' => opts.flags.count = true,
            'l' => opts.flags.files_with_matches = true,
            'E' => opts.extended_regex = true,
            'F' => opts.fixed_strings = true,
            'r' | 'R' => opts.recursive = true,
            _ => {}
        });
    }

    fn add_non_flag_arg(opts: &mut Self, arg: &str) {
        if opts.pattern.is_none() {
            opts.pattern = Some(arg.to_string());
        } else {
            opts.files.push(arg.to_string());
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let pattern = self.pattern.as_deref().unwrap_or("pattern");
        let clean_pattern = pattern.trim_matches('"').trim_matches('\'');

        if self.should_use_find() {
            self.build_find_replacement(clean_pattern)
        } else {
            self.build_where_replacement(clean_pattern)
        }
    }

    const fn should_use_find(&self) -> bool {
        !self.flags.invert_match
            && !self.flags.line_number
            && !self.flags.count
            && !self.flags.files_with_matches
            && !self.extended_regex
            && !self.fixed_strings
            && !self.recursive
            && self.files.is_empty()
    }

    fn build_find_replacement(&self, pattern: &str) -> (String, String) {
        let replacement = format!("find \"{pattern}\"");
        let description = self.build_find_description(pattern);
        (replacement, description)
    }

    fn build_where_replacement(&self, pattern: &str) -> (String, String) {
        let (filter_expr, examples) = self.build_where_filter(pattern);

        let replacement = if self.files.is_empty() {
            format!("lines | {filter_expr}")
        } else {
            format!("open {} | lines | {filter_expr}", self.files.join(" "))
        };

        let description = self.build_where_description(pattern, &examples);
        (replacement, description)
    }

    fn build_find_description(&self, pattern: &str) -> String {
        let mut parts = vec![format!(
            "Use 'find \"{}\"' for simple text search.",
            pattern
        )];

        if self.flags.case_insensitive {
            parts.push(
                "The -i flag is redundant; Nu's 'find' is case-insensitive by default.".to_string(),
            );
        } else {
            parts.push("Nu's 'find' is case-insensitive by default.".to_string());
        }

        parts.push(
            "The 'find' command works on structured data, lists, and strings, making it more \
             versatile than grep."
                .to_string(),
        );

        parts.join(" ")
    }

    fn build_where_filter(&self, pattern: &str) -> (String, Vec<String>) {
        let mut examples = Vec::new();
        let mut filter_parts = Vec::new();

        // Pattern matching
        let pattern_expr = if self.fixed_strings {
            examples.push("fixed string: use 'str contains' for literal matching".to_string());
            format!("$it | str contains \"{pattern}\"")
        } else if self.flags.invert_match {
            examples.push("invert: '!~ pattern' for non-matching lines".to_string());
            format!("$it !~ \"{pattern}\"")
        } else {
            examples.push("regex: '=~ pattern' for regex matching".to_string());
            format!("$it =~ \"{pattern}\"")
        };

        filter_parts.push(format!("where {pattern_expr}"));

        // Additional transformations
        if self.flags.line_number {
            examples.push("line numbers: use 'enumerate' before filtering".to_string());
            filter_parts.insert(0, "enumerate".to_string());
        }

        if self.flags.count {
            examples.push("count: pipe to 'length' to count matches".to_string());
            filter_parts.push("length".to_string());
        }

        (filter_parts.join(" | "), examples)
    }

    fn build_where_description(&self, pattern: &str, examples: &[String]) -> String {
        let mut parts = vec![format!(
            "Use 'lines | where' to filter lines by pattern '{}'.",
            pattern
        )];

        if !examples.is_empty() {
            parts.push(format!("Pipeline stages: {}", examples.join("; ")));
        }

        if self.flags.case_insensitive {
            parts.push(
                "Note: -i flag is redundant in Nu. Use 'str downcase' before filtering for \
                 case-insensitive regex, or 'find' which is case-insensitive by default."
                    .to_string(),
            );
        }

        parts.push(
            "Nushell's 'where' operates on structured data, enabling powerful filtering without \
             text parsing."
                .to_string(),
        );

        parts.join(" ")
    }
}

struct UseBuiltinGrep;

impl DetectFix for UseBuiltinGrep {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_grep"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'find' or 'where' instead of 'grep' for better data handling"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/find.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = context.external_invocations("grep", NOTE);
        // ripgrep
        violations.extend(context.external_invocations("rg", NOTE));
        violations
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = GrepOptions::parse(fix_data.arg_strings.iter().copied());
        let (replacement, description) = opts.to_nushell();

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement {
                span: fix_data.expr_span.into(),
                replacement_text: replacement.into(),
            }],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinGrep;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
