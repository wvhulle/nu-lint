use std::iter;

use nu_lint_analysis::{
    context::LintContext,
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    DeclId, ParseWarning, Span,
    ast::{Argument, Call, Expr, Traverse},
};

use super::MIGRATION_TABLES;

pub struct MigratedDeprecation {
    pub command: &'static str,
    pub flag: Option<&'static str>,
    pub successor: Option<&'static str>,
}

impl MigratedDeprecation {
    #[must_use]
    pub const fn renamed_command(command: &'static str, successor: &'static str) -> Self {
        Self {
            command,
            flag: None,
            successor: Some(successor),
        }
    }

    #[must_use]
    pub const fn renamed_flag(
        command: &'static str,
        flag: &'static str,
        successor: &'static str,
    ) -> Self {
        Self {
            command,
            flag: Some(flag),
            successor: Some(successor),
        }
    }

    #[must_use]
    pub const fn removed_flag(command: &'static str, flag: &'static str) -> Self {
        Self {
            command,
            flag: Some(flag),
            successor: None,
        }
    }
}

#[cfg(test)]
fn every_migration() -> impl Iterator<Item = &'static MigratedDeprecation> {
    MIGRATION_TABLES.iter().copied().flatten()
}

pub struct DeprecatedFlag {
    pub span: Span,
    pub span_with_leading_space: Span,
    pub is_written_long: bool,
}

pub struct MigratedCall {
    pub migration: &'static MigratedDeprecation,
    pub decl_id: DeclId,
    pub head: Span,
    pub flag: Option<DeprecatedFlag>,
    pub warning_span: Span,
}

struct ResolvedMigration {
    migration: &'static MigratedDeprecation,
    decl_id: DeclId,
}

const fn warning_span(warning: &ParseWarning) -> Span {
    let ParseWarning::Deprecated { span, .. } = warning;
    *span
}

fn deprecated_flag(call: &Call, flag_name: &str, context: &LintContext) -> Option<DeprecatedFlag> {
    let signature = context.working_set.get_decl(call.decl_id).signature();
    let signature_flag = signature.get_long_flag(flag_name)?;
    let preceding_token_ends =
        iter::once(call.head.end).chain(call.arguments.iter().map(|arg| arg.span().end));
    preceding_token_ends
        .zip(&call.arguments)
        .find_map(|(preceding_token_end, arg)| match arg {
            Argument::Named((name, _, None)) if name.item == signature_flag.long => {
                Some(DeprecatedFlag {
                    span: name.span,
                    span_with_leading_space: Span::new(preceding_token_end, name.span.end),
                    is_written_long: name.span.len() == signature_flag.long.len() + 2,
                })
            }
            _ => None,
        })
}

fn migrated_call(
    call: &Call,
    resolved: &ResolvedMigration,
    warning_spans: &[Span],
    context: &LintContext,
) -> Option<MigratedCall> {
    let flag = match resolved.migration.flag {
        Some(flag_name) => Some(deprecated_flag(call, flag_name, context)?),
        None => None,
    };
    let warning_span = flag.as_ref().map_or_else(|| call.span(), |flag| flag.span);
    warning_spans
        .contains(&warning_span)
        .then_some(MigratedCall {
            migration: resolved.migration,
            decl_id: call.decl_id,
            head: call.head,
            flag,
            warning_span,
        })
}

#[must_use]
pub fn migrated_calls(
    context: &LintContext,
    migrations: &'static [MigratedDeprecation],
) -> Vec<MigratedCall> {
    let warning_spans: Vec<_> = context
        .working_set
        .parse_warnings
        .iter()
        .map(warning_span)
        .collect();
    let resolved_migrations: Vec<_> = migrations
        .iter()
        .filter_map(|migration| {
            Some(ResolvedMigration {
                migration,
                decl_id: context
                    .working_set
                    .find_decl(migration.command.as_bytes())?,
            })
        })
        .collect();
    let mut calls = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| match &expr.expr {
            Expr::Call(call) => resolved_migrations
                .iter()
                .filter(|resolved| resolved.decl_id == call.decl_id)
                .filter_map(|resolved| migrated_call(call, resolved, &warning_spans, context))
                .collect(),
            _ => vec![],
        },
        &mut calls,
    );
    calls
}

pub struct CommandRename {
    head: Span,
    successor: &'static str,
}

impl CommandRename {
    #[must_use]
    pub fn fix(&self) -> Fix {
        Fix {
            explanation: format!("Rename the command to `{}`", self.successor).into(),
            replacements: vec![Replacement::new(self.head, self.successor)],
        }
    }
}

#[must_use]
pub fn command_renames(
    context: &LintContext,
    migrations: &'static [MigratedDeprecation],
) -> Vec<(Detection, CommandRename)> {
    migrated_calls(context, migrations)
        .into_iter()
        .filter_map(|call| {
            let successor = call.migration.successor?;
            let detection = Detection::from_global_span(
                format!(
                    "`{}` is deprecated in favour of `{successor}`",
                    call.migration.command
                ),
                call.head,
            )
            .with_primary_label("deprecated");
            Some((
                detection,
                CommandRename {
                    head: call.head,
                    successor,
                },
            ))
        })
        .collect()
}

#[must_use]
pub fn migrated_warning_spans(context: &LintContext) -> Vec<Span> {
    MIGRATION_TABLES
        .iter()
        .flat_map(|migrations| migrated_calls(context, migrations))
        .map(|call| call.warning_span)
        .collect()
}

#[cfg(test)]
mod tests;
