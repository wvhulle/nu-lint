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

const HOSTNAME: &str = "sys host | get hostname";

const NOTE: &str = "Nu reads system information directly and returns typed values such as \
                    datetimes, durations and file sizes instead of text that needs parsing.";

fn only_flag(parsed: &ParsedCli, candidates: &[Flag]) -> Option<Flag> {
    let mut present = candidates.iter().filter(|flag| parsed.has(**flag));
    let flag = *present.next()?;
    present.next().is_none().then_some(flag)
}

fn builtin_equivalent(invocation: &ExternalInvocation) -> Option<&'static str> {
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
        "hostname" => Some(HOSTNAME),
        "uname" => match only_flag(&parsed, UNAME_SPEC.flags)? {
            KERNEL_RELEASE => Some("$nu.os-info.kernel_version"),
            NODE_NAME => Some(HOSTNAME),
            _ => Some("$nu.os-info.arch"),
        },
        "uptime" => match only_flag(&parsed, UPTIME_SPEC.flags)? {
            SINCE => Some("sys host | get boot_time"),
            _ => Some("sys host | get uptime"),
        },
        _ => Some("sys mem"),
    }
}

struct SystemInfoToSys;

impl DetectFix for SystemInfoToSys {
    type FixInput<'a> = Replacement;

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
                let equivalent = builtin_equivalent(invocation)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label(format!("'{}' prints text", invocation.name));
                Some((detection, Replacement::new(invocation.span, equivalent)))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Query with '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
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
