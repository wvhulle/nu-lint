use std::iter::Peekable;

use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'date now' for current time, '| date to-timezone <TZ>' for timezone \
                    conversion, and '| format date \"%Y-%m-%d\"' to format. Use '... | into \
                    datetime' to parse strings.";

struct DateOptions {
    fmt: Option<String>,
    utc: bool,
    date_str: Option<String>,
}

impl DateOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self {
            fmt: None,
            utc: false,
            date_str: None,
        };

        let mut iter = args.into_iter().peekable();
        while let Some(text) = iter.next() {
            Self::parse_arg(&mut opts, text, &mut iter);
        }

        opts
    }

    fn parse_arg<'a>(
        opts: &mut Self,
        text: &'a str,
        iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) {
        match text {
            "-u" | "--utc" => opts.utc = true,
            s if s.starts_with("--date=") => {
                opts.date_str = Some(s.to_string());
            }
            "-d" | "--date" => {
                if let Some(next_text) = iter.next() {
                    opts.date_str = Some(next_text.to_string());
                }
            }
            s if s.starts_with('+') => {
                opts.fmt = Some(s.to_string());
            }
            _ => {}
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let mut parts: Vec<String> = Vec::new();
        let mut explanation = vec!["Replace external 'date' with Nushell date pipeline".into()];

        explanation.push("returns a datetime value".to_string());

        if let Some(ds_text) = &self.date_str {
            let trimmed = ds_text.strip_prefix("--date=").unwrap_or(ds_text).trim();
            let ds_quoted = if trimmed.starts_with('\'') || trimmed.starts_with('"') {
                trimmed.to_string()
            } else {
                format!("'{trimmed}'")
            };
            parts.push(format!("{ds_quoted} | into datetime"));
            explanation.push("parse string with 'into datetime'".to_string());
        } else {
            parts.push("date now".to_string());
        }

        if self.utc {
            parts.push("date to-timezone UTC".to_string());
            explanation.push("convert to UTC with 'date to-timezone'".to_string());
        }

        if let Some(f_text) = &self.fmt {
            let rest = &f_text[1..];
            let needs_quotes = !(rest.starts_with('"') || rest.starts_with('\''));
            let fmt_str = if needs_quotes {
                format!("'{rest}'")
            } else {
                rest.to_string()
            };
            parts.push(format!("format date {fmt_str}"));
            explanation.push("format output with 'format date'".to_string());
        }

        if explanation.len() == 2 {
            explanation.push("use 'date now' for current date/time".to_string());
        }

        (parts.join(" | "), explanation.join("; "))
    }
}

struct UseBuiltinDate;

impl DetectFix for UseBuiltinDate {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "date_to_date_now"
    }

    fn short_description(&self) -> &'static str {
        "External `date` replaceable with `date now`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/date_now.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("date", |_, fix_data, ctx| {
            // Only detect simple date usage, not complex formatting or date arithmetic
            let has_complex = fix_data.arg_texts(ctx).any(|text| {
                matches!(
                    text,
                    "-d" | "--date" |           // Parse arbitrary date string
                    "-f" | "--file" |           // Read dates from file
                    "-r" | "--reference" |      // Display file modification time
                    "-s" | "--set" |            // Set system time
                    "--resolution" |            // Show resolution
                    "-I" | "--iso-8601" |       // ISO 8601 (could support)
                    "-R" | "--rfc-email" // RFC 5322 (could support)
                ) || text.starts_with("--date=")
                    || text.starts_with("--file=")
            });
            if has_complex { None } else { Some(NOTE) }
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = DateOptions::parse(fix_data.arg_texts(context));
        let (replacement, explanation) = opts.to_nushell();

        Some(Fix::with_explanation(
            explanation,
            vec![Replacement::new(fix_data.expr_span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &UseBuiltinDate;

#[cfg(test)]
mod tests {
    use super::RULE;

    #[test]
    fn converts_date_command_to_date_now() {
        let source = "^date";
        RULE.assert_fixed_contains(source, "date now");
        RULE.assert_fix_explanation_contains(source, "datetime");
    }

    #[test]
    fn converts_date_with_format_string() {
        // External date formatting like +%Y-%m-%d should still prefer 'date now'
        let source = "^date +%Y-%m-%d";
        RULE.assert_fixed_contains(source, "date now");
        RULE.assert_fix_explanation_contains(source, "datetime");
    }

    #[test]
    fn converts_date_with_utc_flag() {
        // UTC flag doesn't change the recommendation to use 'date now'
        let source = "^date -u";
        RULE.assert_fixed_contains(source, "date now");
        RULE.assert_fix_explanation_contains(source, "datetime");
    }

    #[test]
    fn ignores_builtin_date_now() {
        let source = "date now";
        RULE.assert_ignores(source);
    }

    #[test]
    fn ignores_builtin_date_pipeline() {
        let source = "date now | format date '%Y-%m-%d'";
        RULE.assert_ignores(source);
    }
}
