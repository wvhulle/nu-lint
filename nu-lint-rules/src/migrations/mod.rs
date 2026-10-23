use deprecated_usage::MigratedDeprecation;

pub mod deprecated_usage;
pub mod drop_get_sensitive;
pub mod filter_to_where;
pub mod ignore_errors_to_optional;
pub mod legacy_completer_inputs;
pub mod str_case_full_names;

pub const MIGRATION_TABLES: &[&[MigratedDeprecation]] = &[
    filter_to_where::MIGRATIONS,
    str_case_full_names::MIGRATIONS,
    ignore_errors_to_optional::MIGRATIONS,
    drop_get_sensitive::MIGRATIONS,
];
