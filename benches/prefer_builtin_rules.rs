use std::{fs, hint::black_box};

use criterion::{Criterion, criterion_group, criterion_main};
use nu_cmd_lang::create_default_context;
use nu_command::add_shell_command_context;
use nu_lint::{
    context::LintContext,
    rules::prefer_builtin::{
        cat::rule as prefer_builtin_cat, find::rule as prefer_builtin_find,
        grep::rule as prefer_builtin_grep, head::rule as prefer_builtin_head,
        ls::rule as prefer_builtin_ls, other::rule as prefer_builtin_other,
        sort::rule as prefer_builtin_sort, tail::rule as prefer_builtin_tail,
        uniq::rule as prefer_builtin_uniq,
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

// Quick benchmark using small file
fn bench_prefer_builtin_small(c: &mut Criterion) {
    let source =
        fs::read_to_string("benches/fixtures/small_file.nu").expect("Failed to read small_file.nu");
    let engine_state = add_shell_command_context(create_default_context());

    let mut group = c.benchmark_group("prefer_builtin_small");
    group.sample_size(20);

    // Benchmark just parsing (baseline cost)
    group.bench_function("baseline_parse", |b| {
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

    group.bench_function("ls_rule_only", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            let context = create_lint_context(&source, &engine_state, &working_set, &block);
            let rule = prefer_builtin_ls();
            black_box(rule.check(&context));
        });
    });

    group.bench_function("common_builtin_rules_sequential", |b| {
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

// Medium file benchmark
fn bench_prefer_builtin_medium(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/medium_file.nu")
        .expect("Failed to read medium_file.nu");
    let engine_state = add_shell_command_context(create_default_context());

    let mut group = c.benchmark_group("prefer_builtin_medium");
    group.sample_size(10);

    group.bench_function("ls_rule_only", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            let context = create_lint_context(&source, &engine_state, &working_set, &block);
            black_box(prefer_builtin_ls().check(&context));
        });
    });

    group.bench_function("common_builtin_rules_sequential", |b| {
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

// Small file with dense violations to test actual rule matching work
fn bench_prefer_builtin_violations(c: &mut Criterion) {
    let source = fs::read_to_string("benches/fixtures/with_violations.nu")
        .expect("Failed to read with_violations.nu");
    let engine_state = add_shell_command_context(create_default_context());

    let mut group = c.benchmark_group("prefer_builtin_violations");
    group.sample_size(10);

    group.bench_function("ls_rule_only", |b| {
        b.iter(|| {
            let mut working_set = StateWorkingSet::new(&engine_state);
            let block = parse(&mut working_set, None, black_box(&source).as_bytes(), false);
            let context = create_lint_context(&source, &engine_state, &working_set, &block);
            black_box(prefer_builtin_ls().check(&context));
        });
    });

    group.bench_function("common_builtin_rules_sequential", |b| {
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

criterion_group!(
    benches,
    bench_prefer_builtin_small,
    bench_prefer_builtin_medium,
    bench_prefer_builtin_violations
);
criterion_main!(benches);
