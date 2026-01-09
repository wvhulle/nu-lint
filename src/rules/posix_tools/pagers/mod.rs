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
    filename: Option<String>,
    follow: bool,
}

impl PagerOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();

        for text in args {
            match text {
                "-f" | "--follow" | "-F" => opts.follow = true,
                s if !s.starts_with('-') => opts.filename = Some(s.to_string()),
                _ => {}
            }
        }

        opts
    }

    fn to_nushell(&self) -> (String, String) {
        let (replacement, description) = if self.follow {
            let file = self.filename.as_deref().unwrap_or("file");
            (
                format!("watch {file} {{ open --raw {file} | lines | last 20 }}"),
                "Use 'watch' to monitor file changes. Nu's watch executes a closure when the file \
                 changes, similar to 'tail -f'. Note: this is event-based, not continuous \
                 streaming."
                    .to_string(),
            )
        } else {
            let replacement = self.filename.as_ref().map_or_else(
                || "open --raw | explore".to_string(),
                |file| format!("open --raw {file} | explore"),
            );
            (
                replacement,
                "Use 'open --raw | explore' for interactive viewing. Nu's explore provides \
                 keyboard navigation for data. For structured files (JSON, TOML), use 'open file \
                 | explore' without --raw."
                    .to_string(),
            )
        };
        (replacement, description)
    }
}

struct UseBuiltinPager;

impl DetectFix for UseBuiltinPager {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_pager"
    }

    fn short_description(&self) -> &'static str {
        "Use Nu's 'explore' for interactive viewing or 'watch' for monitoring file changes"
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
        let opts = PagerOptions::parse(fix_data.arg_texts(context));
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

pub static RULE: &dyn Rule = &UseBuiltinPager;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
