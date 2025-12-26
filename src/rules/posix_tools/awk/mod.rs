use crate::{
    LintLevel,
    context::LintContext,
    external_commands::{ExternalCmdFixData},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'where' for filtering rows, 'split column' for field extraction, 'select' \
                    for column projection, or 'each' for row-by-row transformation. Nushell's \
                    structured data pipelines replace awk's text-based approach with typed \
                    columns and native operations.";

fn strip_quotes(s: &str) -> &str {
    let t = s.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        &t[1..t.len() - 1]
    } else {
        t
    }
}

#[derive(Default)]
struct AwkOptions {
    field_separator: Option<String>,
    pattern: Option<String>,
    print_fields: Vec<usize>,
    files: Vec<String>,
    nf_referenced: bool,
    nr_referenced: bool,
}

impl AwkOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();
        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            Self::parse_arg(&mut opts, arg, &mut iter);
        }

        opts
    }

    fn parse_arg<'a>(opts: &mut Self, arg: &'a str, iter: &mut impl Iterator<Item = &'a str>) {
        match arg {
            "-F" => {
                if let Some(sep) = iter.next() {
                    opts.field_separator = Some(strip_quotes(sep).to_string());
                }
            }
            s if s.starts_with("-F") && s.len() > 2 => {
                opts.field_separator = Some(strip_quotes(&s[2..]).to_string());
            }
            "-v" | "-f" => {
                iter.next();
            }
            s if s.starts_with('"') || s.starts_with('\'') => {
                opts.parse_program(strip_quotes(s));
            }
            s if !s.starts_with('-') && !s.contains('{') => {
                opts.files.push(s.to_string());
            }
            s if s.contains('{') => {
                opts.parse_program(s);
            }
            _ => {}
        }
    }

    fn parse_program(&mut self, program: &str) {
        let p = program.trim();

        if let Some((start, end)) = p
            .find('/')
            .and_then(|s| p[s + 1..].find('/').map(|er| (s, s + 1 + er)))
        {
            self.pattern = Some(p[start + 1..end].to_string());
        }

        let body = p
            .trim_start_matches(|c: char| c == '{' || c.is_whitespace())
            .trim_end_matches(|c: char| c == '}' || c.is_whitespace());

        self.extract_print_fields(body);

        if body.contains("NF") {
            self.nf_referenced = true;
        }
        if body.contains("NR") {
            self.nr_referenced = true;
        }
    }

    fn extract_print_fields(&mut self, body: &str) {
        let mut idx = 0;
        while let Some(pos) = body[idx..].find("print $") {
            let field_start = idx + pos + 7;
            let digits: String = body[field_start..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            if let Ok(n) = digits.parse::<usize>() {
                self.add_field_if_valid(n);
            }
            idx = field_start + digits.len();
        }
    }

    fn add_field_if_valid(&mut self, n: usize) {
        if n > 0 && !self.print_fields.contains(&n) {
            self.print_fields.push(n);
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let mut parts: Vec<String> = Vec::new();
        let mut examples: Vec<String> = Vec::new();

        if let Some(file) = self.files.first() {
            parts.push(format!("open --raw {file} | lines"));
        } else {
            parts.push("lines".to_string());
        }

        if let Some(pat) = &self.pattern {
            parts.push(format!("where $it =~ \"{pat}\""));
            examples.push(format!("/{pat}/ pattern: use 'where $it =~ \"{pat}\"'"));
        }

        self.add_field_processing(&mut parts, &mut examples);

        if self.nr_referenced {
            parts.insert(1, "enumerate".to_string());
            examples.push("NR: use 'enumerate' for line numbers".to_string());
        }

        if self.nf_referenced && self.print_fields.is_empty() {
            examples.push("NF: use '($row | columns | length)' for field count".to_string());
        }

        if parts.len() == 1 {
            parts.push("each {|line| $line}".to_string());
        }

        let replacement = parts.join(" | ");
        let description = build_description(&examples);
        (replacement, description)
    }

    fn add_field_processing(&self, parts: &mut Vec<String>, examples: &mut Vec<String>) {
        if self.print_fields.is_empty() && !self.nf_referenced {
            return;
        }

        let sep = self.field_separator.as_deref().unwrap_or(" ");
        let sep_display = if sep == " " { "\" \"" } else { sep };
        parts.push(format!("split column {sep_display}"));
        examples.push(format!("-F{sep}: use 'split column {sep_display}'"));

        if self.print_fields.len() == 1 {
            let col = format!("column{}", self.print_fields[0]);
            parts.push(format!("get {col}"));
            examples.push(format!("${}: use 'get {col}'", self.print_fields[0]));
        } else if self.print_fields.len() > 1 {
            let cols: Vec<String> = self
                .print_fields
                .iter()
                .map(|n| format!("column{n}"))
                .collect();
            parts.push(format!("select {}", cols.join(" ")));
            examples.push("multiple $N: use 'select column1 column2 ...'".to_string());
        }
    }
}

fn build_description(examples: &[String]) -> String {
    let mut parts = vec!["Convert awk to Nushell pipeline.".to_string()];

    if !examples.is_empty() {
        parts.push(format!("Conversions: {}.", examples.join("; ")));
    }

    parts.push(
        "Nushell's structured data replaces awk's $N fields with typed columns, enabling \
         operations like 'where', 'select', 'sort-by' without text parsing."
            .to_string(),
    );

    parts.join(" ")
}

struct UseBuiltinAwk;

impl DetectFix for UseBuiltinAwk {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_awk"
    }

    fn explanation(&self) -> &'static str {
        "Use Nushell pipelines (where/split column/select/each) instead of awk"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/coming_from_bash.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = context.external_invocations("awk", NOTE);
        violations.extend(context.external_invocations("gawk", NOTE));
        violations.extend(context.external_invocations("mawk", NOTE));
        violations
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = AwkOptions::parse(fix_data.arg_strings.iter().copied());
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

pub static RULE: &dyn Rule = &UseBuiltinAwk;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
