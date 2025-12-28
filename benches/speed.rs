use std::{fs, hint::black_box};

use criterion::{Criterion, criterion_group, criterion_main};
use nu_lint::{Config, LintEngine, apply_fixes_iteratively};

fn bench_lint(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/with_violations.nu")
        .expect("Failed to read with_violations.nu");
    let config = Config::default();
    let lint_engine = LintEngine::new(config);

    c.bench_function("lint", |b| {
        b.iter(|| {
            black_box(lint_engine.lint_str(&source));
        });
    });
}

fn bench_fix(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/with_violations.nu")
        .expect("Failed to read with_violations.nu");
    let config = Config::default();
    let lint_engine = LintEngine::new(config);

    c.bench_function("fix", |b| {
        b.iter(|| {
            black_box(apply_fixes_iteratively(&source, &lint_engine));
        });
    });
}

criterion_group!(benches, bench_lint, bench_fix);
criterion_main!(benches);
