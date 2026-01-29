use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'sys disks' to get structured disk usage information. Nu's sys disks \
                    returns a table with name, type, mount, total, free, and removable fields \
                    that you can easily filter and manipulate.";

#[derive(Default)]
struct DfOptions {
    human_readable: bool,
    show_all: bool,
    show_type: bool,
    paths: Vec<String>,
}

impl DfOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();

        for text in args {
            Self::parse_arg(&mut opts, text);
        }

        opts
    }

    fn parse_arg(opts: &mut Self, text: &str) {
        match text {
            "-h" | "--human-readable" => opts.human_readable = true,
            "-a" | "--all" => opts.show_all = true,
            "-T" | "--print-type" => opts.show_type = true,
            s if !s.starts_with('-') => opts.paths.push(s.to_string()),
            _ => {}
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let base = "sys disks";

        let (replacement, description) = if !self.paths.is_empty() {
            let path = &self.paths[0];
            let replacement = format!("{base} | where mount == {path}");
            let description = format!(
                "Use 'sys disks | where mount == {path}' to filter disk information for a \
                 specific mount point. The result is structured data you can further filter or \
                 transform."
            );
            (replacement, description)
        } else if self.show_type {
            let replacement = format!("{base} | select name mount type total free");
            let description = "Use 'sys disks' to get disk information with filesystem types. The \
                               'type' field shows the filesystem type for each disk."
                .to_string();
            (replacement, description)
        } else {
            let replacement = base.to_string();
            let description = "Use 'sys disks' to get structured disk information. Returns a \
                               table with columns for name, type, mount, total, and free space. \
                               Much easier to work with than parsing df's text output."
                .to_string();
            (replacement, description)
        };

        (replacement, description)
    }
}

struct UseSysDisksInsteadOfDf;

impl DetectFix for UseSysDisksInsteadOfDf {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "df_to_sys_disks"
    }

    fn short_description(&self) -> &'static str {
        "`df` replaceable with `sys disks`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sys_disks.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("df", |_, fix_data, ctx| {
            // Only detect common, translatable flags
            let unsupported = fix_data.arg_texts(ctx).any(|text| {
                matches!(
                    text,
                    "-i" | "--inodes" |           // inode info not in sys disks
                    "-B" | "--block-size" |      // custom block size
                    "--output" |                  // custom output format
                    "-x" | "--exclude-type" |    // type exclusion
                    "-t" | "--type" |            // type filtering (use Nu filtering)
                    "--total" |                   // total row
                    "-P" | "--portability" // POSIX format
                )
            });
            if unsupported { None } else { Some(NOTE) }
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = DfOptions::parse(fix_data.arg_texts(context));
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

pub static RULE: &dyn Rule = &UseSysDisksInsteadOfDf;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
