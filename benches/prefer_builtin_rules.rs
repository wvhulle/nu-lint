#![allow(clippy::excessive_nesting)]

use std::{fs, hint::black_box};

use criterion::{Criterion, criterion_group, criterion_main};
use nu_cmd_lang::create_default_context;
use nu_command::add_shell_command_context;
use nu_lint::{
    context::LintContext,
    rules::prefer_builtin::{
        prefer_builtin_cat, prefer_builtin_find, prefer_builtin_grep, prefer_builtin_head,
        prefer_builtin_ls, prefer_builtin_other, prefer_builtin_sort, prefer_builtin_tail,
        prefer_builtin_uniq,
    },
};
use nu_parser::parse;
use nu_protocol::engine::StateWorkingSet;

fn create_lint_context<'a>(
    source: &'a str,
    engine_state: &'a nu_protocol::engine::EngineState,
    working_set: &'a StateWorkingSet<'a>,
    block: &'a nu_protocol::ast::Block,
) -> LintContext<'a> {
    LintContext {
        source,
        file_path: None,
        ast: block,
        engine_state,
        working_set,
    }
}

// Quick benchmark - runs in ~5-10 seconds
fn bench_quick(c: &mut Criterion) {
    let source =
        fs::read_to_string("benches/fixtures/small_file.nu").expect("Failed to read small_file.nu");
    let engine_state = add_shell_command_context(create_default_context());

    let mut group = c.benchmark_group("quick");
    group.sample_size(10); // Smaller sample for speed

    // Benchmark just parsing (baseline cost)
    group.bench_function("parse_only", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            black_box(create_lint_context(
                &source,
                &engine_state,
                &working_set,
                &block,
            ));
        });
    });

    // Benchmark a single rule
    group.bench_function("single_rule", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            let context = create_lint_context(&source, &engine_state, &working_set, &block);
            let rule = prefer_builtin_ls();
            black_box(rule.check(&context));
        });
    });

    // Benchmark all 9 rules to show overhead
    group.bench_function("all_9_rules", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            let context = create_lint_context(&source, &engine_state, &working_set, &block);
            black_box(prefer_builtin_ls().check(&context));
            black_box(prefer_builtin_cat().check(&context));
            black_box(prefer_builtin_grep().check(&context));
            black_box(prefer_builtin_find().check(&context));
            black_box(prefer_builtin_head().check(&context));
            black_box(prefer_builtin_tail().check(&context));
            black_box(prefer_builtin_sort().check(&context));
            black_box(prefer_builtin_uniq().check(&context));
            black_box(prefer_builtin_other().check(&context));
        });
    });

    group.finish();
}

// Medium file benchmark for comparison
fn bench_medium(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/medium_file.nu")
        .expect("Failed to read medium_file.nu");
    let engine_state = add_shell_command_context(create_default_context());

    let mut group = c.benchmark_group("medium");
    group.sample_size(10);

    group.bench_function("single_rule", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            let context = create_lint_context(&source, &engine_state, &working_set, &block);
            black_box(prefer_builtin_ls().check(&context));
        });
    });

    group.bench_function("all_9_rules", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            let context = create_lint_context(&source, &engine_state, &working_set, &block);
            black_box(prefer_builtin_ls().check(&context));
            black_box(prefer_builtin_cat().check(&context));
            black_box(prefer_builtin_grep().check(&context));
            black_box(prefer_builtin_find().check(&context));
            black_box(prefer_builtin_head().check(&context));
            black_box(prefer_builtin_tail().check(&context));
            black_box(prefer_builtin_sort().check(&context));
            black_box(prefer_builtin_uniq().check(&context));
            black_box(prefer_builtin_other().check(&context));
        });
    });

    group.finish();
}

// Violations-heavy file to test actual work
fn bench_with_violations(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/with_violations.nu")
        .expect("Failed to read with_violations.nu");
    let engine_state = add_shell_command_context(create_default_context());

    let mut group = c.benchmark_group("violations");
    group.sample_size(10);

    group.bench_function("single_rule", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            let context = create_lint_context(&source, &engine_state, &working_set, &block);
            black_box(prefer_builtin_ls().check(&context));
        });
    });

    group.bench_function("all_9_rules", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            let context = create_lint_context(&source, &engine_state, &working_set, &block);
            black_box(prefer_builtin_ls().check(&context));
            black_box(prefer_builtin_cat().check(&context));
            black_box(prefer_builtin_grep().check(&context));
            black_box(prefer_builtin_find().check(&context));
            black_box(prefer_builtin_head().check(&context));
            black_box(prefer_builtin_tail().check(&context));
            black_box(prefer_builtin_sort().check(&context));
            black_box(prefer_builtin_uniq().check(&context));
            black_box(prefer_builtin_other().check(&context));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_quick, bench_medium, bench_with_violations);
criterion_main!(benches);
