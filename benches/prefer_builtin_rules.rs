use std::{fs, hint::black_box};

use criterion::{Criterion, criterion_group, criterion_main};
use nu_lint::{Config, LintEngine};

// Quick benchmark using small file
fn bench_prefer_builtin_small(c: &mut Criterion) {
    let source =
        fs::read_to_string("benches/fixtures/small_file.nu").expect("Failed to read small_file.nu");

    let mut group = c.benchmark_group("prefer_builtin_small");
    group.sample_size(20);

    group.bench_function("full_lint_engine", |b| {
        let config = Config::default();
        let lint_engine = LintEngine::new(config);
        b.iter(|| {
            black_box(lint_engine.lint_source(&source, None));
        });
    });

    group.finish();
}

// Medium file benchmark
fn bench_prefer_builtin_medium(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/medium_file.nu")
        .expect("Failed to read medium_file.nu");

    let mut group = c.benchmark_group("prefer_builtin_medium");
    group.sample_size(10);

    group.bench_function("full_lint_engine", |b| {
        let config = Config::default();
        let lint_engine = LintEngine::new(config);
        b.iter(|| {
            black_box(lint_engine.lint_source(&source, None));
        });
    });

    group.finish();
}

// Small file with dense violations to test actual rule matching work
fn bench_prefer_builtin_violations(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/with_violations.nu")
        .expect("Failed to read with_violations.nu");

    let mut group = c.benchmark_group("prefer_builtin_violations");
    group.sample_size(10);

    group.bench_function("full_lint_engine", |b| {
        let config = Config::default();
        let lint_engine = LintEngine::new(config);
        b.iter(|| {
            black_box(lint_engine.lint_source(&source, None));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_prefer_builtin_small,
    bench_prefer_builtin_medium,
    bench_prefer_builtin_violations
);
criterion_main!(benches);
