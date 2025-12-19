use nu_protocol::ast::ExternalArgument;

use crate::{
    context::LintContext,
    effect::{
        CommonEffect, is_dangerous_path, is_unvalidated_variable, matches_long_flag,
        matches_short_flag,
    },
};

/// Things that may happen at runtime for external commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternEffect {
    /// Effect that is common between built-in and external commands.
    CommonEffect(CommonEffect),
    /// Silent, does not produce useful output
    NoDataInStdout,
    /// This command modifies the file system
    ModifiesFileSystem,
}

pub fn has_external_side_effect(
    command_name: &str,
    side_effect: ExternEffect,
    context: &LintContext,
    args: &[ExternalArgument],
) -> bool {
    log::debug!("Checking external side effect '{side_effect:?}' for command '{command_name}'");

    let result = EXTERNAL_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .and_then(|(_, effects)| {
            effects
                .iter()
                .find(|(effect, _)| *effect == side_effect)
                .map(|(_, predicate)| {
                    log::debug!("Checking external predicate for side effect '{side_effect:?}'");
                    predicate(context, args)
                })
        })
        .unwrap_or(false);

    if result {
        log::debug!("External predicate matched for side effect '{side_effect:?}'");
    } else {
        log::debug!(
            "No matching external side effect '{side_effect:?}' found for command '{command_name}'"
        );
    }

    result
}

pub fn has_external_recursive_flag(args: &[ExternalArgument], context: &LintContext) -> bool {
    args.iter().any(|arg| {
        let arg_text = extract_external_arg_text(arg, context);
        matches!(
            arg_text,
            text if text.contains("-r")
                || text.contains("--recursive")
                || text.contains("-rf")
                || text.contains("-fr")
                || text.contains("--force")
        )
    })
}

pub fn is_external_command_safe(command_name: &str) -> bool {
    EXTERNAL_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .is_some_and(|(_, effects)| {
            !effects.iter().any(|(effect, _)| {
                *effect == ExternEffect::CommonEffect(CommonEffect::LikelyErrors)
            })
        })
}

pub fn external_command_has_no_output(command_name: &str) -> bool {
    EXTERNAL_COMMAND_SIDE_EFFECTS
        .iter()
        .find(|(name, _)| *name == command_name)
        .is_some_and(|(_, effects)| {
            effects
                .iter()
                .any(|(effect, _)| *effect == ExternEffect::NoDataInStdout)
        })
}

pub fn extract_external_arg_text<'a>(arg: &ExternalArgument, context: &'a LintContext) -> &'a str {
    match arg {
        ExternalArgument::Regular(expr) | ExternalArgument::Spread(expr) => {
            std::str::from_utf8(context.working_set.get_span_contents(expr.span))
                .unwrap_or("")
        }
    }
}

pub type ExternalSideEffectPredicate = fn(&LintContext, &[ExternalArgument]) -> bool;

const fn always(_context: &LintContext, _args: &[ExternalArgument]) -> bool {
    true
}

fn has_flag(args: &[ExternalArgument], context: &LintContext, patterns: &[&str]) -> bool {
    let matches_pattern = |arg_text: &str, pattern: &str| match pattern.strip_prefix("--") {
        Some(_) => matches_long_flag(arg_text, pattern),
        None => pattern
            .strip_prefix('-')
            .filter(|rest| rest.len() == 1)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|flag_char| {
                matches_long_flag(arg_text, pattern) || matches_short_flag(arg_text, flag_char)
            }),
    };

    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|arg_text| {
            patterns
                .iter()
                .any(|pattern| matches_pattern(arg_text, pattern))
        })
}

fn external_rm_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
        || has_external_recursive_flag(args, context)
}

fn external_mv_cp_is_dangerous(context: &LintContext, args: &[ExternalArgument]) -> bool {
    args.iter()
        .map(|arg| extract_external_arg_text(arg, context))
        .any(|path| is_dangerous_path(path) || is_unvalidated_variable(path))
}

fn curl_modifies_fs(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(args, context, &["-o", "--output", "-O", "--remote-name"])
}

fn tar_modifies_fs(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(
        args,
        context,
        &["-x", "--extract", "--get", "-c", "--create"],
    )
}

fn sed_has_inplace(context: &LintContext, args: &[ExternalArgument]) -> bool {
    has_flag(args, context, &["-i", "--in-place"])
}

pub const EXTERNAL_COMMAND_SIDE_EFFECTS: &[(
    &str,
    &[(ExternEffect, ExternalSideEffectPredicate)],
)] = &[
    // Nushell std lib assert commands (parsed as external when std not loaded)
    ("assert", &[(ExternEffect::NoDataInStdout, always)]),
    (
        "rm",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                external_rm_is_dangerous,
            ),
        ],
    ),
    (
        "mv",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                external_mv_cp_is_dangerous,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "cp",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::NoDataInStdout, always),
            (
                ExternEffect::CommonEffect(CommonEffect::Dangerous),
                external_mv_cp_is_dangerous,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "tar",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, tar_modifies_fs),
        ],
    ),
    ("echo", &[]),
    (
        "zip",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
        ],
    ),
    (
        "unzip",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, always),
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
        ],
    ),
    (
        "ssh",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
    (
        "curl",
        &[
            (
                ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
                always,
            ),
            (ExternEffect::ModifiesFileSystem, curl_modifies_fs),
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
        ],
    ),
    (
        "find",
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
        "xargs",
        &[(
            ExternEffect::CommonEffect(CommonEffect::LikelyErrors),
            always,
        )],
    ),
];

#[cfg(test)]
mod tests {
    use nu_protocol::ast::Expr;

    use super::*;

    fn with_external_args<F, R>(source: &str, f: F) -> R
    where
        F: for<'b> FnOnce(&LintContext<'b>, &[ExternalArgument]) -> R,
    {
        LintContext::test_with_parsed_source(source, |context| {
            let args = context
                .ast
                .pipelines
                .first()
                .and_then(|pipeline| pipeline.elements.first())
                .and_then(|element| match &element.expr.expr {
                    Expr::ExternalCall(_, args) => Some(args.as_ref()),
                    _ => None,
                })
                .unwrap_or(&[]);
            f(&context, args)
        })
    }

    #[test]
    fn test_curl_without_output_flag_does_not_modify_filesystem() {
        with_external_args("curl https://example.com", |context, args| {
            assert!(
                !has_external_side_effect("curl", ExternEffect::ModifiesFileSystem, context, args),
                "curl without output flag should not modify filesystem"
            );
        });
    }

    #[test]
    fn test_curl_with_short_output_flag_modifies_filesystem() {
        with_external_args("curl -o output.txt https://example.com", |context, args| {
            assert!(
                has_external_side_effect("curl", ExternEffect::ModifiesFileSystem, context, args),
                "curl with -o flag should modify filesystem"
            );
        });
    }

    #[test]
    fn test_curl_with_long_output_flag_modifies_filesystem() {
        with_external_args(
            "curl --output output.txt https://example.com",
            |context, args| {
                assert!(
                    has_external_side_effect(
                        "curl",
                        ExternEffect::ModifiesFileSystem,
                        context,
                        args
                    ),
                    "curl with --output flag should modify filesystem"
                );
            },
        );
    }

    #[test]
    fn test_curl_with_remote_name_modifies_filesystem() {
        with_external_args("curl -O https://example.com/file.txt", |context, args| {
            assert!(
                has_external_side_effect("curl", ExternEffect::ModifiesFileSystem, context, args),
                "curl with -O flag should modify filesystem"
            );
        });
    }

    #[test]
    fn test_tar_list_does_not_modify_filesystem() {
        with_external_args("tar -t -f archive.tar", |context, args| {
            assert!(
                !has_external_side_effect("tar", ExternEffect::ModifiesFileSystem, context, args),
                "tar -t (list) should not modify filesystem"
            );
        });
    }

    #[test]
    fn test_tar_extract_modifies_filesystem() {
        with_external_args("tar -x -f archive.tar", |context, args| {
            assert!(
                has_external_side_effect("tar", ExternEffect::ModifiesFileSystem, context, args),
                "tar -x (extract) should modify filesystem"
            );
        });
    }

    #[test]
    fn test_tar_create_modifies_filesystem() {
        with_external_args("tar -c -f archive.tar files/", |context, args| {
            assert!(
                has_external_side_effect("tar", ExternEffect::ModifiesFileSystem, context, args),
                "tar -c (create) should modify filesystem"
            );
        });
    }

    #[test]
    fn test_tar_create_combined_flags_modifies_filesystem() {
        with_external_args("tar czf backup.tar.gz folder/", |context, args| {
            assert!(
                has_external_side_effect("tar", ExternEffect::ModifiesFileSystem, context, args),
                "tar czf (create with compression) should modify filesystem"
            );
        });
    }

    #[test]
    fn test_sed_without_inplace_does_not_modify_filesystem() {
        with_external_args("sed 's/foo/bar/' file.txt", |context, args| {
            assert!(
                !has_external_side_effect("sed", ExternEffect::ModifiesFileSystem, context, args),
                "sed without -i should not modify filesystem"
            );
        });
    }

    #[test]
    fn test_sed_with_inplace_modifies_filesystem() {
        with_external_args("sed -i 's/foo/bar/' file.txt", |context, args| {
            assert!(
                has_external_side_effect("sed", ExternEffect::ModifiesFileSystem, context, args),
                "sed with -i should modify filesystem"
            );
        });
    }

    #[test]
    fn test_sed_without_inplace_is_not_dangerous() {
        with_external_args("sed 's/foo/bar/' file.txt", |context, args| {
            assert!(
                !has_external_side_effect(
                    "sed",
                    ExternEffect::CommonEffect(CommonEffect::Dangerous),
                    context,
                    args
                ),
                "sed without -i should not be dangerous"
            );
        });
    }

    #[test]
    fn test_sed_with_inplace_is_dangerous() {
        with_external_args("sed -i 's/foo/bar/' file.txt", |context, args| {
            assert!(
                has_external_side_effect(
                    "sed",
                    ExternEffect::CommonEffect(CommonEffect::Dangerous),
                    context,
                    args
                ),
                "sed with -i should be dangerous"
            );
        });
    }
}
