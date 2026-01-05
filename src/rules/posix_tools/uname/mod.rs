use crate::{
    LintLevel,
    context::{ExternalCmdFixData, LintContext},
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "Use 'sys host' to get system information. Nu's sys host returns structured \
                    data with fields like name, os_version, kernel_version, hostname, and uptime.";

#[derive(Default)]
struct UnameOptions {
    all: bool,
    kernel_name: bool,
    kernel_release: bool,
    kernel_version: bool,
    machine: bool,
    os_name: bool,
}

impl UnameOptions {
    fn parse<'a>(args: impl IntoIterator<Item = &'a str>) -> Self {
        let mut opts = Self::default();

        for arg in args {
            Self::parse_arg(&mut opts, arg);
        }

        opts
    }

    fn parse_arg(opts: &mut Self, arg: &str) {
        match arg {
            "-a" | "--all" => opts.all = true,
            "-s" | "--kernel-name" => opts.kernel_name = true,
            "-r" | "--kernel-release" => opts.kernel_release = true,
            "-v" | "--kernel-version" => opts.kernel_version = true,
            "-m" | "--machine" => opts.machine = true,
            "-o" | "--operating-system" => opts.os_name = true,
            _ => {}
        }
    }

    fn to_nushell(&self) -> (String, String) {
        let (replacement, description) = if self.all {
            (
                "sys host".to_string(),
                "Use 'sys host' to get all system information as a structured record. Fields \
                 include name, kernel_version, os_version, hostname, and more."
                    .to_string(),
            )
        } else if self.kernel_release || self.kernel_version {
            (
                "sys host | get kernel_version".to_string(),
                "Use 'sys host | get kernel_version' to get the kernel version.".to_string(),
            )
        } else if self.kernel_name || self.os_name {
            (
                "sys host | get name".to_string(),
                "Use 'sys host | get name' to get the operating system name.".to_string(),
            )
        } else if self.machine {
            (
                "sys host | get name".to_string(),
                "Use 'sys host | get name' for system information. For architecture, use \
                 $nu.os-info.arch."
                    .to_string(),
            )
        } else {
            (
                "sys host | get name".to_string(),
                "Use 'sys host | get name' to get the operating system name. For more details, \
                 use 'sys host' to get all system information."
                    .to_string(),
            )
        };

        (replacement, description)
    }
}

struct UseSysHostInsteadOfUname;

impl DetectFix for UseSysHostInsteadOfUname {
    type FixInput<'a> = ExternalCmdFixData<'a>;

    fn id(&self) -> &'static str {
        "use_sys_host_instead_of_uname"
    }

    fn explanation(&self) -> &'static str {
        "Use Nu's 'sys host' command instead of 'uname' for system information"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sys_host.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_external_with_validation("uname", |_, _| Some(NOTE))
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let opts = UnameOptions::parse(fix_data.arg_strings(_context));
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

pub static RULE: &dyn Rule = &UseSysHostInsteadOfUname;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
