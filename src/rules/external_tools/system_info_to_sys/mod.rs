use nu_protocol::Span;

use crate::{
    LintLevel,
    ast::external::{CliSpec, ExternalInvocation, Flag, ParsedCli},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const KERNEL_RELEASE: Flag = Flag::switch('r', "kernel-release");
const NODE_NAME: Flag = Flag::switch('n', "nodename");
const MACHINE: Flag = Flag::switch('m', "machine");
const SINCE: Flag = Flag::switch('s', "since");
const PRETTY: Flag = Flag::switch('p', "pretty");
const BYTES: Flag = Flag::switch('b', "bytes");
const HUMAN: Flag = Flag::switch('h', "human");

static NO_FLAGS: CliSpec = CliSpec {
    flags: &[],
    numeric_shorthand: None,
};

static UNAME_SPEC: CliSpec = CliSpec {
    flags: &[KERNEL_RELEASE, NODE_NAME, MACHINE],
    numeric_shorthand: None,
};

static UPTIME_SPEC: CliSpec = CliSpec {
    flags: &[SINCE, PRETTY],
    numeric_shorthand: None,
};

static FREE_SPEC: CliSpec = CliSpec {
    flags: &[BYTES, HUMAN],
    numeric_shorthand: None,
};

const NOTE: &str = "Nu reads system information directly and returns typed values such as \
                    datetimes, durations and file sizes instead of text that needs parsing.";

fn only_flag(parsed: &ParsedCli, candidates: &[Flag]) -> Option<Flag> {
    let mut present = candidates.iter().filter(|flag| parsed.has(**flag));
    let flag = *present.next()?;
    present.next().is_none().then_some(flag)
}

#[derive(Clone, Copy)]
enum SystemQuery {
    Hostname,
    KernelRelease,
    Machine,
    BootTime,
    Uptime,
    Memory,
}

impl SystemQuery {
    fn of(invocation: &ExternalInvocation) -> Option<Self> {
        let spec = match invocation.name {
            "hostname" => &NO_FLAGS,
            "uname" => &UNAME_SPEC,
            "uptime" => &UPTIME_SPEC,
            _ => &FREE_SPEC,
        };
        let parsed = invocation
            .parse(spec)
            .filter(|parsed| parsed.operands.is_empty())?;
        match invocation.name {
            "hostname" => Some(Self::Hostname),
            "uname" => match only_flag(&parsed, UNAME_SPEC.flags)? {
                KERNEL_RELEASE => Some(Self::KernelRelease),
                NODE_NAME => Some(Self::Hostname),
                _ => Some(Self::Machine),
            },
            "uptime" => match only_flag(&parsed, UPTIME_SPEC.flags)? {
                SINCE => Some(Self::BootTime),
                _ => Some(Self::Uptime),
            },
            _ => Some(Self::Memory),
        }
    }

    const fn nu_expression(self) -> &'static str {
        match self {
            Self::Hostname => "sys host | get hostname",
            Self::KernelRelease => "$nu.os-info.kernel_version",
            Self::Machine => "$nu.os-info.arch",
            Self::BootTime => "sys host | get boot_time",
            Self::Uptime => "sys host | get uptime",
            Self::Memory => "sys mem",
        }
    }
}

pub struct FixData {
    span: Span,
    query: SystemQuery,
}

struct SystemInfoToSys;

impl DetectFix for SystemInfoToSys {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "system_info_to_sys"
    }

    fn short_description(&self) -> &'static str {
        "System information command replaceable with `sys` or `$nu.os-info`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sys.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["hostname", "uname", "uptime", "free"])
            .iter()
            .filter_map(|invocation| {
                let query = SystemQuery::of(invocation)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label(format!("'{}' prints text", invocation.name));
                Some((
                    detection,
                    FixData {
                        span: invocation.span,
                        query,
                    },
                ))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = fix_data.query.nu_expression();
        Some(Fix {
            explanation: format!("Query with '{replacement}'").into(),
            replacements: vec![Replacement::new(fix_data.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &SystemInfoToSys;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
