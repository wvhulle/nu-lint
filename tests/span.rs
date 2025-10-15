/// Test that BP009-AST produces valid spans (not Span::unknown)
#[test]
fn test_bp009_ast_has_valid_span() {
    use nu_lint::{Config, LintEngine};
    use std::path::Path;

    let engine = LintEngine::new(Config::default());
    let test_file = Path::new("tests/nu/BP009-AST.nu");

    let violations = engine.lint_file(test_file).expect("Failed to lint file");

    // Should find at least one violation for BP009-AST
    let ast_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "BP009-AST")
        .collect();

    assert!(
        !ast_violations.is_empty(),
        "Should detect BP009-AST violation"
    );

    // Verify the span is not unknown (unknown spans have start == end == 0)
    for violation in &ast_violations {
        assert!(
            violation.span.start != 0 || violation.span.end != 0,
            "Violation span should not be unknown (0, 0), got ({}, {})",
            violation.span.start,
            violation.span.end
        );
        assert!(
            violation.span.end > violation.span.start,
            "Span end ({}) should be greater than start ({})",
            violation.span.end,
            violation.span.start
        );
    }
}
