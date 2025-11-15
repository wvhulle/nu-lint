use std::{fs, hint::black_box};

use criterion::{Criterion, criterion_group, criterion_main};
use nu_lint::{Config, LintEngine};

// Benchmark using small real-world module
fn bench_lint_small_file(c: &mut Criterion) {
    let source =
        fs::read_to_string("benches/fixtures/small_file.nu").expect("Failed to read small_file.nu");

    let mut group = c.benchmark_group("lint_small_file");
    group.sample_size(20);

    group.bench_function("full_lint_engine", |b| {
        let config = Config::default();
        let lint_engine = LintEngine::new(config);
        b.iter(|| {
            black_box(lint_engine.lint_str(&source));
        });
    });

    group.finish();
}

// Benchmark using medium real-world module
fn bench_lint_medium_file(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/medium_file.nu")
        .expect("Failed to read medium_file.nu");

    let mut group = c.benchmark_group("lint_medium_file");
    group.sample_size(10);

    group.bench_function("full_lint_engine", |b| {
        let config = Config::default();
        let lint_engine = LintEngine::new(config);
        b.iter(|| {
            black_box(lint_engine.lint_str(&source));
        });
    });

    group.finish();
}

// Benchmark using file with many violations to test rule matching overhead
fn bench_lint_with_violations(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/with_violations.nu")
        .expect("Failed to read with_violations.nu");

    let mut group = c.benchmark_group("lint_with_violations");
    group.sample_size(10);

    group.bench_function("full_lint_engine", |b| {
        let config = Config::default();
        let lint_engine = LintEngine::new(config);
        b.iter(|| {
            black_box(lint_engine.lint_str(&source));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lint_small_file,
    bench_lint_medium_file,
    bench_lint_with_violations
);
criterion_main!(benches);
