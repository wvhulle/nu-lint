use nu_protocol::Span;

use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'open --raw | explore' for interactive file viewing, or 'watch' for \
                    monitoring file changes (like tail -f). Nu's explore provides structured data \
                    navigation.";

#[derive(Default)]
struct PagerOptions {
    span: Span,
    filename: Option<String>,
    follow: bool,
}

impl PagerOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>, span: Span) -> Self {
        let mut opts = Self {
            span,
            ..Self::default()
        };

        for text in args {
            match text {
                "-f" | "--follow" | "-F" => opts.follow = true,
                s if !s.starts_with('-') => opts.filename = Some(s.to_string()),
                _ => {}
            }
        }

        opts
    }

    fn to_nushell(&self) -> Option<Fix> {
        if self.follow {
            let file = self.filename.as_deref()?;

            return Some(Fix {
                explanation: "Use 'watch' to monitor file changes. Nu's watch executes a closure \
                              when the file changes, similar to 'tail -f'. Note: this is \
                              event-based, not continuous streaming."
                    .into(),
                replacements: vec![Replacement::new(
                    self.span,
                    format!("watch {file} {{ open --raw {file} | lines | last 20 }}"),
                )],
            });
        }

        let replacement = self.filename.as_ref().map_or_else(
            || "explore".to_string(),
            |file| format!("open --raw {file} | explore"),
        );

        Some(Fix {
            explanation: "Use 'open --raw | explore' for interactive viewing. Nu's explore \
                          provides keyboard navigation for data. For structured files (JSON, \
                          TOML), use 'open file | explore' without --raw."
                .into(),
            replacements: vec![Replacement::new(self.span, replacement)],
        })
    }
}

struct UseBuiltinPager;

impl DetectFix for UseBuiltinPager {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "pager_to_explore"
    }

    fn short_description(&self) -> &'static str {
        "Pager replaceable with `explore`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/explore.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        // Pagers (less/more) have good Nu alternatives
        // Most usage is straightforward and translates well
        let mut violations = context.detect_external_with_validation("less", |_, _, _| Some(NOTE));
        violations.extend(context.detect_external_with_validation("more", |_, _, _| Some(NOTE)));
        violations
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = PagerOptions::parse(fix_data.arg_texts(context), fix_data.expr_span);

        opts.to_nushell()
    }
}

pub static RULE: &dyn Rule = &UseBuiltinPager;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
