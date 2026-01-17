use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, has_flag},
};
use crate::{context::LintContext, effect::CommonEffect};

fn curl_modifies_fs(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(args, context, &["-o", "--output", "-O", "--remote-name"])
}

fn curl_has_streaming(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(
        args,
        context,
        &["-o", "--output", "-O", "--remote-name", "--progress-bar"],
    )
}

fn rsync_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(
        args,
        context,
        &[
            "--delete",
            "--delete-before",
            "--delete-after",
            "--delete-during",
        ],
    )
}

pub const COMMANDS: &[CommandEffects] = &[
    (
        "curl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, curl_modifies_fs),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, curl_has_streaming),
        ],
    ),
    (
        "wget",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "rsync",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                rsync_is_dangerous,
            ),
        ],
    ),
    (
        "scp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "sftp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "ssh",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "ftp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    // Network diagnostics
    (
        "ping",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "traceroute",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "mtr",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::SlowStreamingOutput, always),
        ],
    ),
    (
        "dig",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "nslookup",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    (
        "host",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
        ],
    ),
    // Network config
    (
        "ip",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            // (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "ifconfig",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "netstat",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "ss",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "lsof",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Potentially dangerous network tools
    (
        "nc",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "netcat",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "ncat",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "socat",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesNetworkState, always),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    // Firewall
    (
        "iptables",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "nft",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "ufw",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
    (
        "firewall-cmd",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::CommonEffect(CommonEffect::Dangerous), always),
        ],
    ),
];
