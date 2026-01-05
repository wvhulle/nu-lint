use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'sys mem' to get structured memory information. Nu's sys mem returns a \
                    record with total, free, used, and available memory fields that you can \
                    easily filter and manipulate.";

#[derive(Default)]
struct FreeOptions {
    human_readable: bool,
    bytes: bool,
    kilobytes: bool,
    megabytes: bool,
    gigabytes: bool,
    wide: bool,
    total: bool,
}

impl FreeOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();

        for text in args {
            Self::parse_arg(&mut opts, text);
        }

        opts
    }

    fn parse_arg(opts: &mut Self, text: &str) {
        match text {
            "-h" | "--human" => opts.human_readable = true,
            "-b" | "--bytes" => opts.bytes = true,
            "-k" | "--kibi" | "--kilo" => opts.kilobytes = true,
            "-m" | "--mebi" | "--mega" => opts.megabytes = true,
            "-g" | "--gibi" | "--giga" => opts.gigabytes = true,
            "-w" | "--wide" => opts.wide = true,
            "-t" | "--total" => opts.total = true,
            _ => {}
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let base = "sys mem";

        let (replacement, description) =
            if self.bytes || self.kilobytes || self.megabytes || self.gigabytes {
                let unit = if self.bytes {
                    "bytes"
                } else if self.kilobytes {
                    "KB"
                } else if self.megabytes {
                    "MB"
                } else {
                    "GB"
                };

                let replacement = base.to_string();
                let description = format!(
                    "Use 'sys mem' to get memory info. All fields are already in human-readable \
                     format. To convert to {unit}, use field conversion like: sys mem | get total \
                     | into int"
                );
                (replacement, description)
            } else if self.total {
                let replacement = base.to_string();
                let description = "Use 'sys mem' to get total memory information. All totals are \
                                   included in the returned record."
                    .to_string();
                (replacement, description)
            } else {
                let replacement = base.to_string();
                let description = "Use 'sys mem' to get structured memory information with fields \
                                   like 'total', 'free', 'used', and 'available'. Much easier to \
                                   work with than parsing free's text output."
                    .to_string();
                (replacement, description)
            };

        (replacement, description)
    }
}

struct UseSysMemInsteadOfFree;

impl DetectFix for UseSysMemInsteadOfFree {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_sys_mem_instead_of_free"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'sys mem' command instead of 'free' for memory information"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sys_mem.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("free", |_, _, _| Some(NOTE))
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = FreeOptions::parse(fix_data.arg_texts(context));
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

pub static RULE: &dyn Rule = &UseSysMemInsteadOfFree;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
