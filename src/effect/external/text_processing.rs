use nu_protocol::ast::ExternalArgument;

use super::{
    CommandEffects, ExternEffect,
    predicates::{always, has_flag},
};
use crate::{context::LintContext, effect::CommonEffect};

fn sed_has_inplace(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(args, context, &["-i", "--in-place"])
}

pub const COMMANDS: &[CommandEffects] = &[
    (
        "cat",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "head",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "tail",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "grep",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "awk",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "sed",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, sed_has_inplace),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                sed_has_inplace,
            ),
        ],
    ),
    (
        "sort",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "uniq",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "wc",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "cut",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "tr",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "xargs",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "diff",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "patch",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    // Modern alternatives
    (
        "rg",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "fd",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    ("bat", &[]),
    (
        "sd",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Data processing
    (
        "jq",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "yq",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "mlr",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "csvtool",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    // Pagers
    ("less", &[(ExternEffect::NoDataInStdout, always)]),
    ("more", &[(ExternEffect::NoDataInStdout, always)]),
];
