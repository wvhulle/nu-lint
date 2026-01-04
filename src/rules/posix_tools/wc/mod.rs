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
        "use_builtin_wc"
    }

    fn explanation(&self) -> &'static str {
        "Prefer 'length' over external wc"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/length.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("wc", |_, args| {
            // Only reliably translate -l (line count) to 'lines | length'
            // Don't detect -c (bytes), -m (chars), -w (words), or -L (max line length)
            let has_complex = args.iter().any(|arg| {
                matches!(
                    *arg,
                    "-c" | "--bytes" |          // Byte count
                    "-m" | "--chars" |          // Character count (different from str length)
                    "-w" | "--words" |          // Word count
                    "-L" | "--max-line-length" | // Longest line
                    "--files0-from" // Read from file list
                )
            });
            // Only detect if it's -l or no flags (default includes line count)
            let has_line_flag = args.iter().any(|arg| *arg == "-l" || *arg == "--lines");
            let has_only_files = args.iter().all(|arg| !arg.starts_with('-'));

            if has_complex {
                None
            } else if has_line_flag || has_only_files {
                Some(NOTE)
            } else {
                None
            }
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let (replacement, description) = if fix_data.arg_strings.iter().copied().any(|x| x == "-l")
        {
            (
                "lines | length".to_string(),
                "Use 'lines | length' to count lines in a file".to_string(),
            )
        } else {
            (
                "length".to_string(),
                "Use 'length' for item count or 'str length' for character count".to_string(),
            )
        };
        Some(Fix::with_explanation(
            description,
            vec![Replacement::new(fix_data.expr_span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &UseBuiltinWc;

#[cfg(test)]
mod tests {
    use super::RULE;

    #[test]
    fn converts_wc_lines_to_lines_length() {
        let source = "^wc -l";
        RULE.assert_fixed_contains(source, "lines | length");
        RULE.assert_fix_explanation_contains(source, "count");
    }
}
