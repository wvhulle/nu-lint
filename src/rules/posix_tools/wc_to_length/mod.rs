use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'length' for item count or 'str length' for character count.";

struct UseBuiltinWc;

impl DetectFix for UseBuiltinWc {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "wc_to_length"
    }

    fn short_description(&self) -> &'static str {
        "`wc` replaceable with `length`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/length.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("wc", |_, fix_data, ctx| {
            // Only reliably translate -l (line count) to 'lines | length'
            // Don't detect -c (bytes), -m (chars), -w (words), or -L (max line length)
            let arg_texts: Vec<&str> = fix_data.arg_texts(ctx).collect();
            let has_complex = arg_texts.iter().any(|text| {
                matches!(
                    *text,
                    "-c" | "--bytes" |          // Byte count
                    "-m" | "--chars" |          // Character count (different from str length)
                    "-w" | "--words" |          // Word count
                    "-L" | "--max-line-length" | // Longest line
                    "--files0-from" // Read from file list
                )
            });
            // Only detect if it's -l or no flags (default includes line count)
            let has_line_flag = arg_texts
                .iter()
                .any(|text| *text == "-l" || *text == "--lines");
            let has_only_files = arg_texts.iter().all(|text| !text.starts_with('-'));

            if has_complex {
                None
            } else if has_line_flag || has_only_files {
                Some(NOTE)
            } else {
                None
            }
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let arg_texts: Vec<&str> = fix_data.arg_texts(context).collect();
        let counts_lines = arg_texts
            .iter()
            .any(|text| *text == "-l" || *text == "--lines");
        let files: Vec<&&str> = arg_texts
            .iter()
            .filter(|text| !text.starts_with('-'))
            .collect();

        if files.len() > 1 {
            return None;
        }

        let counting = if counts_lines {
            "lines | length"
        } else {
            "length"
        };
        let replacement = match files.first() {
            Some(file) => format!("open --raw {file} | {counting}"),
            None => counting.to_string(),
        };

        Some(Fix {
            explanation: if counts_lines {
                "Use 'lines | length' to count lines".into()
            } else {
                "Use 'length' for item count or 'str length' for character count".into()
            },
            replacements: vec![Replacement::new(fix_data.expr_span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinWc;

#[cfg(test)]
mod generated_fix;
