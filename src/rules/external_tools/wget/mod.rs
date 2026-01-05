use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'http get URL | save file' to download files. This provides structured \
                    data handling and better pipeline integration than wget.";

#[derive(Default)]
struct WgetOptions {
    url: Option<String>,
    output_file: Option<String>,
}

impl WgetOptions {
    fn parse_wget<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        args.into_iter()
            .fold(
                (Self::default(), None::<&str>),
                |(mut opts, expecting), arg| match (expecting, arg) {
                    (Some("-O" | "--output-document"), file) => {
                        opts.output_file = Some(file.to_string());
                        (opts, None)
                    }
                    (None, "-O" | "--output-document") => (opts, Some(arg)),
                    (None, s) if !s.starts_with('-') && opts.url.is_none() => {
                        opts.url = Some(s.to_string());
                        (opts, None)
                    }
                    _ => (opts, None),
                },
            )
            .0
    }

    fn to_nushell(&self) -> (String, String) {
        let url = self.url.as_deref().unwrap_or("URL");
        let mut replacement = format!("http get {url}");
        if let Some(file) = &self.output_file {
            replacement = format!("{replacement} | save {file}");
        }
        let description = if self.output_file.is_some() {
            "Replace wget with 'http get | save'. Downloads return structured data that can be \
             processed before saving."
                .to_string()
        } else {
            "Replace wget with 'http get'. Use '| save <file>' to persist downloads. Nushell's \
             http returns structured data and integrates with pipelines."
                .to_string()
        };
        (replacement, description)
    }
}

struct UseBuiltinWget;

impl DetectFix for UseBuiltinWget {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_builtin_wget"
    }

    fn explanation(&self) -> &'static str {
        "Use 'http get | save' instead of wget"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/http_get.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("wget", |_, args| {
            // Don't detect complex wget features
            let has_complex = args.iter().any(|arg| {
                matches!(
                    *arg,
                    "--mirror" | "-m" |           // Mirroring
                    "--recursive" | "-r" |        // Recursive download
                    "--span-hosts" | "-H" |       // Span hosts
                    "--page-requisites" | "-p" |  // Download page requisites
                    "--convert-links" | "-k" |    // Convert links
                    "--backup-converted" | "-K" | // Backup converted files
                    "--reject" | "-R" |           // Reject patterns
                    "--accept" | "-A" |           // Accept patterns
                    "--level" | "-l" |            // Recursion depth
                    "--quota" | "-Q" |            // Download quota
                    "--wait" | "-w" |             // Wait between requests
                    "--random-wait" |             // Random wait
                    "--no-parent" | "-np" |       // Don't ascend to parent
                    "--timestamping" | "-N" |     // Timestamping
                    "--continue" | "-c" // Continue download (could support)
                )
            });
            if has_complex { None } else { Some(NOTE) }
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = WgetOptions::parse_wget(fix_data.arg_strings(_context));
        let (replacement, description) = opts.to_nushell();

        Some(Fix {
            explanation: description.into(),
            replacements: vec![Replacement::new(fix_data.expr_span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &UseBuiltinWget;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
