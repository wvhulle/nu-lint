use nu_protocol::{
    Span,
    ast::{Block, Expr, Expression, Pipeline},
};

use crate::{ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation};

const SAFE_EXTERNAL_COMMANDS: &[&str] = &[
    "echo", "printf", "true", "false", "yes", "seq", "ls", "date", "uptime", "cal", "whoami", "id",
    "hostname", "uname", "arch", "pwd", "basename", "dirname", "realpath", "readlink", "env",
    "printenv", "tr", "cut", "paste", "column", "fmt", "fold", "expand", "unexpand", "bc", "dc",
    "expr", "mktemp", "git",
];

fn is_safe_command(cmd: &str) -> bool {
    SAFE_EXTERNAL_COMMANDS.contains(&cmd)
}

fn get_external_command(expr: &Expression, context: &LintContext) -> Option<String> {
    if let Expr::ExternalCall(head, _args) = &expr.expr {
        let head_text = context.source[head.span.start..head.span.end].to_string();
        if !is_safe_command(&head_text) {
            return Some(head_text);
        }
    }
    None
}

fn pipeline_has_only_simple_processing(pipeline: &Pipeline, context: &LintContext) -> bool {
    const SIMPLE_COMMANDS: &[&str] = &["lines", "str trim", "split"];

    pipeline.elements.iter().skip(1).all(|element| {
        if let Expr::Call(call) = &element.expr.expr {
            let name = call.get_call_name(context);
            SIMPLE_COMMANDS.iter().any(|cmd| name.starts_with(cmd)) || name == "where" // Simple where filters are okay
        } else {
            true
        }
    })
}

fn is_wrapped_in_try(expr_span: Span, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let mut try_spans = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            matches!(&expr.expr, Expr::Call(call)
            if call.is_call_to_command("try", context))
            .then_some(expr.span)
            .into_iter()
            .collect()
        },
        &mut try_spans,
    );

    try_spans
        .iter()
        .any(|try_span| try_span.contains_span(expr_span))
}

fn is_wrapped_in_complete(pipeline: &Pipeline, context: &LintContext) -> bool {
    pipeline.elements.iter().any(|element| {
        matches!(&element.expr.expr, Expr::Call(call)
            if call.is_call_to_command("complete", context))
    })
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Violation> {
    if pipeline.elements.len() <= 1 {
        return None;
    }

    let first_element = &pipeline.elements[0];
    let external_cmd = get_external_command(&first_element.expr, context)?;

    if !pipeline_has_only_simple_processing(pipeline, context) {
        return None;
    }

    if is_wrapped_in_try(first_element.expr.span, context) {
        return None;
    }

    if is_wrapped_in_complete(pipeline, context) {
        return None;
    }

    let message = format!(
        "External command '{external_cmd}' in simple pipeline should be wrapped in 'try' block"
    );

    let suggestion = "Wrap in 'try {{ ... }}' to prevent script termination on error";

    Some(
        Violation::new_dynamic(
            "prefer_try_for_simple_external_commands",
            message,
            first_element.expr.span,
        )
        .with_suggestion_static(suggestion),
    )
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<Violation>) {
    for pipeline in &block.pipelines {
        violations.extend(check_pipeline(pipeline, context));

        use nu_protocol::ast::Traverse;
        for element in &pipeline.elements {
            let mut blocks = Vec::new();
            element.expr.flat_map(
                context.working_set,
                &|expr| match &expr.expr {
                    Expr::Block(block_id)
                    | Expr::Closure(block_id)
                    | Expr::Subexpression(block_id) => {
                        vec![*block_id]
                    }
                    _ => vec![],
                },
                &mut blocks,
            );

            for &block_id in &blocks {
                let nested_block = context.working_set.get_block(block_id);
                check_block(nested_block, context, violations);
            }
        }
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();
    check_block(context.ast, context, &mut violations);
    violations
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_try_for_simple_external_commands",
        "Simple external command pipelines should use 'try' for error handling",
        check,
    )
}

#[cfg(test)]
mod detect_bad {
    use super::rule;

    #[test]
    fn test_external_with_lines() {
        let bad_code = r"^git status | lines";
        rule().assert_violation(bad_code);
    }

    #[test]
    fn test_external_with_grep() {
        let bad_code = r#"^grep "pattern" file.txt | lines"#;
        rule().assert_violation(bad_code);
    }

    #[test]
    fn test_external_with_simple_where() {
        let bad_code = r#"^find . -name "*.nu" | lines | where $it =~ "test""#;
        rule().assert_violation(bad_code);
    }

    #[test]
    fn test_external_with_str_trim() {
        let bad_code = r"^cat file.txt | str trim";
        rule().assert_violation(bad_code);
    }

    #[test]
    fn test_external_with_split() {
        let bad_code = r"^date | split row ' '";
        rule().assert_violation(bad_code);
    }

    #[test]
    fn test_multiple_simple_commands() {
        let bad_code = r"^git branch | lines | str trim";
        rule().assert_violation(bad_code);
    }
}

#[cfg(test)]
mod ignore_good {
    use super::rule;

    #[test]
    fn test_wrapped_in_try() {
        let good_code = r"try { ^git status | lines }";
        rule().assert_no_violations(good_code);
    }

    #[test]
    fn test_using_complete() {
        let good_code = r"let result = (^git status | complete)";
        rule().assert_no_violations(good_code);
    }

    #[test]
    fn test_single_external_command() {
        let good_code = r"^git status";
        rule().assert_no_violations(good_code);
    }

    #[test]
    fn test_safe_command() {
        let good_code = r"^echo 'test' | lines";
        rule().assert_no_violations(good_code);
    }

    #[test]
    fn test_with_data_processing() {
        // This should be caught by prefer_complete_for_external_commands instead
        let good_code = r"^curl https://api.example.com | from json";
        rule().assert_no_violations(good_code);
    }

    #[test]
    fn test_nested_in_subexpression_with_try() {
        let good_code = r"let x = (try { ^git status | lines })";
        rule().assert_no_violations(good_code);
    }

    #[test]
    fn test_do_ignore_wrapper() {
        let good_code = r"do -i { ^git status | lines }";
        rule().assert_no_violations(good_code);
    }

    #[test]
    fn test_complex_pipeline_not_simple() {
        // Complex processing - should not trigger this rule
        let good_code = r"^curl https://api.example.com | lines | each { |line| $line | parse '{key}: {value}' }";
        rule().assert_no_violations(good_code);
    }
}
