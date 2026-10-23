use std::{
    env, fs,
    hint::black_box,
    path::{Path, PathBuf},
};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use nu_lint::{
    Config, LintEngine, LintLevel, all_rule_ids, apply_fixes_iteratively, collect_nu_files_from_dir,
};
use rayon::prelude::*;

const FIXTURE_NAMES: [&str; 3] = ["small_file", "medium_file", "with_violations"];
const NU_SCRIPTS_DIR_VARIABLE: &str = "NU_SCRIPTS_DIR";
const PARSE_ONLY_BENCH_ID: &str = "parse_only";

fn read_fixture_source(fixture_name: &str) -> String {
    let fixture_path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "benches",
        "fixtures",
        &format!("{fixture_name}.nu"),
    ]
    .iter()
    .collect();
    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", fixture_path.display()))
}

fn read_corpus_sources(corpus_dir: &Path) -> Vec<String> {
    collect_nu_files_from_dir(corpus_dir)
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .collect()
}

fn config_with_only_rule(enabled_rule_id: Option<&str>) -> Config {
    let mut config = Config {
        rules: all_rule_ids()
            .map(|rule_id| (rule_id.to_string(), LintLevel::Off))
            .collect(),
        ..Config::default()
    };
    if let Some(rule_id) = enabled_rule_id {
        config.rules.insert(rule_id.to_string(), LintLevel::Warning);
    }
    config
}

fn lint_all_sources(lint_engine: &LintEngine, sources: &[String]) -> usize {
    sources
        .par_iter()
        .map(|source| lint_engine.lint_str(source).len())
        .sum()
}

fn bench_fixture_lint_and_fix(c: &mut Criterion) {
    let lint_engine = LintEngine::new(Config::default());
    let mut group = c.benchmark_group("fixtures");
    for fixture_name in FIXTURE_NAMES {
        let source = read_fixture_source(fixture_name);
        group.bench_with_input(
            BenchmarkId::new("lint", fixture_name),
            &source,
            |b, source| {
                b.iter(|| black_box(lint_engine.lint_str(source)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("fix", fixture_name),
            &source,
            |b, source| {
                b.iter(|| black_box(apply_fixes_iteratively(source, &lint_engine)));
            },
        );
    }
    group.finish();
}

fn bench_each_rule_on_nu_scripts(c: &mut Criterion) {
    let Some(corpus_dir) = env::var_os(NU_SCRIPTS_DIR_VARIABLE) else {
        eprintln!(
            "Skipping per_rule benchmarks: set {NU_SCRIPTS_DIR_VARIABLE} to a nu_scripts checkout \
             (the nix devshell sets it)"
        );
        return;
    };
    let sources = read_corpus_sources(Path::new(&corpus_dir));
    let mut group = c.benchmark_group("per_rule");
    group.sample_size(10);

    let parse_only_engine = LintEngine::new(config_with_only_rule(None));
    group.bench_function(PARSE_ONLY_BENCH_ID, |b| {
        b.iter(|| black_box(lint_all_sources(&parse_only_engine, &sources)));
    });

    for rule_id in all_rule_ids() {
        let single_rule_engine = LintEngine::new(config_with_only_rule(Some(rule_id)));
        group.bench_function(rule_id, |b| {
            b.iter(|| black_box(lint_all_sources(&single_rule_engine, &sources)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_fixture_lint_and_fix,
    bench_each_rule_on_nu_scripts
);
criterion_main!(benches);
