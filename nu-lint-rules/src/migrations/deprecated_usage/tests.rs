use nu_lint_analysis::engine::default_engine_state;
use nu_protocol::{DeprecationEntry, DeprecationType, Signature};

use super::{MigratedDeprecation, every_migration};
use crate::{
    migrations::{
        drop_get_sensitive, filter_to_where, ignore_errors_to_optional, str_case_full_names,
    },
    upstream::nu_deprecated,
};

const EVERY_MIGRATED_FORM: &str = r"
[1 2] | filter {|x| $x > 1}
'nu' | str upcase | str downcase
{a: 1} | get -i a
[[a]; [1]] | select --ignore-errors a | reject -i a
{a: 1} | get --sensitive a
";

fn upstream_signature(command: &str) -> Option<Signature> {
    let engine_state = default_engine_state();
    let decl_id = engine_state.find_decl(command.as_bytes(), &[])?;
    Some(engine_state.get_decl(decl_id).signature())
}

fn upstream_deprecations(command: &str) -> Vec<DeprecationEntry> {
    let engine_state = default_engine_state();
    engine_state
        .find_decl(command.as_bytes(), &[])
        .map_or_else(Vec::new, |decl_id| {
            engine_state.get_decl(decl_id).deprecation_info()
        })
}

fn is_deprecated_upstream(migration: &MigratedDeprecation) -> bool {
    upstream_deprecations(migration.command)
        .iter()
        .any(|entry| match (&entry.ty, migration.flag) {
            (DeprecationType::Command, None) => true,
            (DeprecationType::Flag(flag), Some(migrated_flag)) => flag == migrated_flag,
            _ => false,
        })
}

fn successor_exists_upstream(migration: &MigratedDeprecation) -> bool {
    match (migration.flag, migration.successor) {
        (_, None) => true,
        (None, Some(successor_command)) => upstream_signature(successor_command).is_some(),
        (Some(_), Some(successor_flag)) => upstream_signature(migration.command)
            .is_some_and(|signature| signature.get_long_flag(successor_flag).is_some()),
    }
}

#[test]
fn every_migration_matches_an_upstream_deprecation() {
    let stale_migrations: Vec<_> = every_migration()
        .filter(|migration| !is_deprecated_upstream(migration))
        .map(|migration| (migration.command, migration.flag))
        .collect();
    assert!(
        stale_migrations.is_empty(),
        "no longer deprecated upstream, retire their migration rules: {stale_migrations:?}"
    );
}

#[test]
fn every_successor_exists_upstream() {
    let missing_successors: Vec<_> = every_migration()
        .filter(|migration| !successor_exists_upstream(migration))
        .map(|migration| (migration.command, migration.successor))
        .collect();
    assert!(
        missing_successors.is_empty(),
        "successors missing upstream: {missing_successors:?}"
    );
}

#[test]
fn each_migrated_warning_is_reported_by_exactly_one_rule() {
    filter_to_where::RULE.assert_count(EVERY_MIGRATED_FORM, 1);
    str_case_full_names::RULE.assert_count(EVERY_MIGRATED_FORM, 2);
    ignore_errors_to_optional::RULE.assert_count(EVERY_MIGRATED_FORM, 3);
    drop_get_sensitive::RULE.assert_count(EVERY_MIGRATED_FORM, 1);
    nu_deprecated::RULE.assert_ignores(EVERY_MIGRATED_FORM);
}
