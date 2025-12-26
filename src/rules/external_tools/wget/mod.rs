use crate::{
    LintLevel,
    context::LintContext,
    external_commands::{ExternalCmdFixData, detect_external_commands, external_args_slices},
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
    type FixInput = ExternalCmdFixData;

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

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        detect_external_commands(context, "wget", NOTE)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput) -> Option<Fix> {
        let opts = WgetOptions::parse_wget(external_args_slices(&fix_data.args, context));
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
