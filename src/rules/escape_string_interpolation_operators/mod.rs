use nu_protocol::ast::{Block, Boolean, Call, Expr, Operator};
use regex::Regex;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

const PROBLEMATIC_BOOLEAN_OPERATORS: &[&str] = &["not", "and", "or", "in"];

const LITERAL_ERROR_MESSAGE_PATTERNS: &[&str] = &[
    "not found",
    "not available",
    "not accessible",
    "not supported",
    "and more",
    "and another",
    "and others",
    "and so on",
    "or else",
    "or more",
    "or less",
    "or other",
    "access denied",
    "failed to",
    "unable to",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectionSensitivity {
    Conservative,
    Balanced,
    Aggressive,
}

const INCOMPLETE_EXPRESSION_PATTERNS: &[&str] = &[
    r"^\s*(\+|\-|\*|\/|==|!=|>=|<=|>|<)\s*$",
    r"^\s*\w+\s+(\+|\-|\*|\/|==|!=|>=|<=|>|<)\s*$",
    r"^\s*(if|while|for)\s+\w+\s*$",
];

/// Common words that appear in natural language but unlikely to be commands
const COMMON_NATURAL_LANGUAGE_WORDS: &[&str] = &[
    "some",
    "text",
    "here",
    "manual",
    "review",
    "needed",
    "this",
    "that",
    "with",
    "note",
    "message",
    "description",
    "example",
];

fn extract_content_from_parentheses(source: &str) -> &str {
    source.trim_start_matches('(').trim_end_matches(')')
}

fn extract_leading_word(content: &str) -> &str {
    content.split_whitespace().next().unwrap_or("")
}

/// Detection categories for problematic expressions
#[derive(Debug, PartialEq)]
enum DangerousInterpolationPattern {
    BooleanOperator(String),
    ErrorMessage(String),
    IncompleteExpression(String),
    ComplexBooleanLogic(String),
}

fn detect_string_based_problems(content: &str) -> Option<DangerousInterpolationPattern> {
    detect_string_based_problems_with_sensitivity(content, DetectionSensitivity::Aggressive)
}

fn detect_string_based_problems_with_sensitivity(
    content: &str,
    sensitivity: DetectionSensitivity,
) -> Option<DangerousInterpolationPattern> {
    let trimmed = content.trim();

    if contains_variable_reference(trimmed) {
        return None;
    }

    let leading_word = extract_leading_word(trimmed);
    if PROBLEMATIC_BOOLEAN_OPERATORS.contains(&leading_word) {
        return Some(DangerousInterpolationPattern::BooleanOperator(
            leading_word.to_string(),
        ));
    }

    if allows_error_patterns(sensitivity) {
        for pattern in LITERAL_ERROR_MESSAGE_PATTERNS {
            if trimmed.starts_with(pattern) {
                return Some(DangerousInterpolationPattern::ErrorMessage(
                    (*pattern).to_string(),
                ));
            }
        }
    }

    if allows_incomplete_expressions(sensitivity) {
        if let Some(incomplete_pattern) = detect_incomplete_expression(trimmed) {
            return Some(DangerousInterpolationPattern::IncompleteExpression(
                incomplete_pattern,
            ));
        }

        if let Some(complex_logic) = detect_complex_boolean_logic(trimmed) {
            return Some(DangerousInterpolationPattern::ComplexBooleanLogic(
                complex_logic,
            ));
        }
    }

    None
}

fn contains_variable_reference(content: &str) -> bool {
    content.contains('$')
}

fn allows_error_patterns(sensitivity: DetectionSensitivity) -> bool {
    !matches!(sensitivity, DetectionSensitivity::Conservative)
}

fn allows_incomplete_expressions(sensitivity: DetectionSensitivity) -> bool {
    matches!(sensitivity, DetectionSensitivity::Aggressive)
}

fn detect_incomplete_expression(content: &str) -> Option<String> {
    for pattern_str in INCOMPLETE_EXPRESSION_PATTERNS {
        if let Ok(regex) = Regex::new(pattern_str)
            && regex.is_match(content)
        {
            return Some(content.to_string());
        }
    }
    None
}

fn detect_complex_boolean_logic(content: &str) -> Option<String> {
    let operator_count: usize = PROBLEMATIC_BOOLEAN_OPERATORS
        .iter()
        .map(|op| content.matches(op).count())
        .sum();

    let word_count = content.split_whitespace().count();
    let is_complex = operator_count > 1 || (operator_count == 1 && word_count > 3);
    let has_expression_indicators = content.contains('(')
        || content.contains('>')
        || content.contains('<')
        || content.contains('=');

    (is_complex && !has_expression_indicators).then(|| content.to_string())
}
fn contains_variables(expr: &nu_protocol::ast::Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::Var(_) => true,
        Expr::FullCellPath(path) => contains_variables(&path.head, context),
        Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block_contains_variables(block, context)
        }
        Expr::BinaryOp(left, _, right) => {
            contains_variables(left, context) || contains_variables(right, context)
        }
        Expr::UnaryNot(inner) => contains_variables(inner, context),
        _ => false,
    }
}

/// Check if a block contains variable references
fn block_contains_variables(block: &Block, context: &LintContext) -> bool {
    block.pipelines.iter().any(|pipeline| {
        pipeline
            .elements
            .iter()
            .any(|element| contains_variables(&element.expr, context))
    })
}

/// Analyze an AST expression to detect problematic patterns
fn analyze_ast_expression(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
    sensitivity: DetectionSensitivity,
) -> Option<DangerousInterpolationPattern> {
    match &expr.expr {
        // VALID: Variable references and safe expressions
        // ANALYZE: Subexpressions - check their contents
        Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            analyze_subexpression_block(block, context, sensitivity)
        }

        // ANALYZE: Full cell paths - check the head
        Expr::FullCellPath(cell_path) => {
            analyze_ast_expression(&cell_path.head, context, sensitivity)
        }

        // PROBLEMATIC: Standalone operators
        Expr::Operator(op) => Some(analyze_standalone_operator(*op)),

        // ANALYZE: Unary not expressions
        Expr::UnaryNot(inner) => analyze_unary_not(inner, context, sensitivity),

        // ANALYZE: Binary operations
        Expr::BinaryOp(left, op, right) => {
            analyze_binary_operation(left, op, right, context, sensitivity)
        }

        // ANALYZE: Function calls (including calls to boolean operators)
        Expr::Call(call) => analyze_function_call(call, context, sensitivity),

        // ANALYZE: External calls (like "and y", "or z", "not found")
        Expr::ExternalCall(head, args) => analyze_external_call(head, args, context, sensitivity),

        // DEFAULT: Other expressions are likely valid
        _ => None,
    }
}

/// Analyze a subexpression block for problematic patterns
fn analyze_subexpression_block(
    block: &Block,
    context: &LintContext,
    sensitivity: DetectionSensitivity,
) -> Option<DangerousInterpolationPattern> {
    // Check each pipeline in the block
    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            if let Some(pattern) = analyze_ast_expression(&element.expr, context, sensitivity) {
                return Some(pattern);
            }
        }
    }
    None
}

/// Analyze standalone operators
fn analyze_standalone_operator(op: Operator) -> DangerousInterpolationPattern {
    match op {
        Operator::Boolean(Boolean::And | Boolean::Or) => {
            DangerousInterpolationPattern::BooleanOperator(format!("{op:?}").to_lowercase())
        }
        _ => DangerousInterpolationPattern::IncompleteExpression(format!(
            "standalone operator {op:?}"
        )),
    }
}

/// Analyze unary not expressions
fn analyze_unary_not(
    _inner: &nu_protocol::ast::Expression,
    _context: &LintContext,
    sensitivity: DetectionSensitivity,
) -> Option<DangerousInterpolationPattern> {
    allows_error_patterns(sensitivity)
        .then(|| DangerousInterpolationPattern::BooleanOperator("not".to_string()))
}

/// Analyze binary operations for problematic patterns
fn analyze_binary_operation(
    left: &nu_protocol::ast::Expression,
    op: &nu_protocol::ast::Expression,
    right: &nu_protocol::ast::Expression,
    context: &LintContext,
    sensitivity: DetectionSensitivity,
) -> Option<DangerousInterpolationPattern> {
    if let Expr::Operator(operator) = &op.expr {
        match operator {
            // Boolean operators that might be intended as literal text
            Operator::Boolean(Boolean::And | Boolean::Or) => {
                // If neither operand contains variables, it's likely meant as literal text
                if !contains_variables(left, context) && !contains_variables(right, context) {
                    Some(DangerousInterpolationPattern::BooleanOperator(
                        format!("{operator:?}").to_lowercase(),
                    ))
                } else {
                    None
                }
            }
            // Check for incomplete binary operations in aggressive mode
            _ if matches!(sensitivity, DetectionSensitivity::Aggressive) => {
                if is_incomplete_binary_op(left, right, context) {
                    Some(DangerousInterpolationPattern::IncompleteExpression(
                        "incomplete binary operation".to_string(),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Check if a binary operation appears incomplete
fn is_incomplete_binary_op(
    left: &nu_protocol::ast::Expression,
    right: &nu_protocol::ast::Expression,
    _context: &LintContext,
) -> bool {
    // Very basic check - in a real implementation you might want more sophisticated
    // logic
    matches!(left.expr, Expr::Nothing) || matches!(right.expr, Expr::Nothing)
}

/// Analyze function calls for problematic patterns
fn analyze_function_call(
    call: &Call,
    context: &LintContext,
    sensitivity: DetectionSensitivity,
) -> Option<DangerousInterpolationPattern> {
    let decl_name = context.working_set.get_decl(call.decl_id).name();

    if matches!(decl_name, "and" | "or" | "not") {
        Some(DangerousInterpolationPattern::BooleanOperator(
            decl_name.to_string(),
        ))
    } else if allows_error_patterns(sensitivity)
        && matches!(decl_name, "failed" | "denied" | "error" | "missing")
        && call.arguments.is_empty()
    {
        Some(DangerousInterpolationPattern::ErrorMessage(
            decl_name.to_string(),
        ))
    } else {
        None
    }
}

/// Analyze external calls for problematic patterns
fn analyze_external_call(
    head: &nu_protocol::ast::Expression,
    args: &[nu_protocol::ast::ExternalArgument],
    context: &LintContext,
    sensitivity: DetectionSensitivity,
) -> Option<DangerousInterpolationPattern> {
    // Check if the head expression is a problematic command name
    if let Expr::GlobPattern(pattern, _) = &head.expr {
        // Check for boolean operator commands that are likely meant as literal text
        if matches!(pattern.as_str(), "and" | "or" | "not") {
            return Some(DangerousInterpolationPattern::BooleanOperator(
                pattern.clone(),
            ));
        }

        // Check for incomplete operator expressions (operators with no or few
        // arguments)
        if matches!(sensitivity, DetectionSensitivity::Aggressive)
            && matches!(
                pattern.as_str(),
                "==" | "!=" | ">" | "<" | ">=" | "<=" | "+" | "-" | "*" | "/"
            )
            && args.is_empty()
        {
            return Some(DangerousInterpolationPattern::IncompleteExpression(
                format!("standalone operator '{pattern}'"),
            ));
        }

        // Check for common error patterns
        for error_pattern in LITERAL_ERROR_MESSAGE_PATTERNS {
            if pattern.starts_with(error_pattern.split_whitespace().next().unwrap_or("")) {
                return Some(DangerousInterpolationPattern::ErrorMessage(pattern.clone()));
            }
        }

        // Check if this looks like plain text rather than a command
        // Use working_set to check if the command actually exists
        if !args.is_empty() && looks_like_plain_text(pattern.as_str(), args, context) {
            return Some(DangerousInterpolationPattern::ComplexBooleanLogic(format!(
                "plain text: {pattern}"
            )));
        }
    }

    None
}

/// Check if an external call looks like plain text rather than a command
fn looks_like_plain_text(
    head: &str,
    args: &[nu_protocol::ast::ExternalArgument],
    context: &LintContext,
) -> bool {
    // Need at least 2 arguments to consider it plain text
    if args.len() < 2 {
        return false;
    }

    // Check if this is a known Nushell command
    if context.working_set.find_decl(head.as_bytes()).is_some() {
        return false; // It's a known command, not plain text
    }

    // If the head is a common natural language word, it's likely plain text
    if COMMON_NATURAL_LANGUAGE_WORDS.contains(&head) {
        return true;
    }

    // Check if any arguments look like flags or special syntax (command-like)
    let has_command_syntax = args.iter().any(|arg| {
        let nu_protocol::ast::ExternalArgument::Regular(expr) = arg else {
            return false;
        };
        let Expr::String(s) = &expr.expr else {
            return false;
        };
        s.starts_with('-') || s.starts_with('$') || s.starts_with('/')
    });

    // If has command-like syntax, it's probably a command
    if has_command_syntax {
        return false;
    }

    // Multiple words without command syntax and not a known command = likely plain
    // text
    true
}

// Note: is_dangerous_subexpression has been replaced by analyze_ast_expression
// which provides better semantic analysis of AST nodes

fn is_valid_interpolation(
    expr: &nu_protocol::ast::Expression,
    _source: &str,
    context: &LintContext,
) -> bool {
    match &expr.expr {
        Expr::Var(_) => true,
        Expr::Subexpression(_) => {
            // For subexpressions, use AST-based analysis but also fall back to string-based
            // analysis to maintain compatibility
            if let Some(_pattern) =
                analyze_ast_expression(expr, context, DetectionSensitivity::Aggressive)
            {
                false // Found a problematic pattern via AST
            } else {
                // If AST doesn't find it problematic, check string-based patterns too
                true // Let the main analysis function handle the fallback
            }
        }
        Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
            Expr::Var(_) => true,
            Expr::Subexpression(_) => {
                // Use AST-based analysis for cell path heads
                analyze_ast_expression(&cell_path.head, context, DetectionSensitivity::Aggressive)
                    .is_none()
            }
            _ => false,
        },
        _ => false,
    }
}

fn create_violation(
    span: nu_protocol::Span,
    pattern: DangerousInterpolationPattern,
) -> RuleViolation {
    let (message, suggestion) = match pattern {
        DangerousInterpolationPattern::BooleanOperator(op) => (
            format!(
                "Unescaped parentheses with boolean operator '{op}' in string interpolation will \
                 cause runtime error"
            ),
            "Escape literal parentheses with backslashes: \\(...\\)".to_string(),
        ),
        DangerousInterpolationPattern::ErrorMessage(pattern) => (
            format!(
                "Error message pattern '{pattern}' in unescaped parentheses will cause runtime \
                 error"
            ),
            "Escape literal parentheses with backslashes: \\(...\\)".to_string(),
        ),
        DangerousInterpolationPattern::IncompleteExpression(expr) => (
            format!(
                "Incomplete expression '{expr}' in string interpolation will cause parse error"
            ),
            "Complete the expression or escape as literal text: \\(...\\)".to_string(),
        ),
        DangerousInterpolationPattern::ComplexBooleanLogic(expr) => (
            format!("Complex boolean expression '{expr}' likely intended as literal text"),
            "Escape literal parentheses with backslashes: \\(...\\)".to_string(),
        ),
    };

    RuleViolation::new_dynamic("escape_string_interpolation_operators", message, span)
        .with_suggestion_dynamic(suggestion)
}

fn check_string_interpolation(
    exprs: &[nu_protocol::ast::Expression],
    span: nu_protocol::Span,
    context: &LintContext,
) -> Option<RuleViolation> {
    exprs
        .iter()
        .filter(|expr| !matches!(expr.expr, Expr::String(_)))
        .map(|expr| (expr, &context.source[expr.span.start..expr.span.end]))
        .filter(|(_, source)| source.starts_with('('))
        .find_map(|(expr, source)| {
            if is_valid_interpolation(expr, source, context) {
                None
            } else if let Some(pattern) =
                analyze_ast_expression(expr, context, DetectionSensitivity::Aggressive)
            {
                Some(create_violation(span, pattern))
            } else {
                let string_content = extract_content_from_parentheses(source);
                detect_string_based_problems(string_content)
                    .map(|pattern| create_violation(span, pattern))
            }
        })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::StringInterpolation(exprs) = &expr.expr
            && let Some(violation) = check_string_interpolation(exprs, expr.span, ctx)
        {
            vec![violation]
        } else {
            vec![]
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "escape_string_interpolation_operators",
        RuleCategory::ErrorHandling,
        Severity::Error,
        "Detect unescaped parentheses with operator keywords in string interpolations that cause \
         runtime errors",
        check,
    )
}

// NOTE: Future enhancement - the DetectionSensitivity enum is ready for
// integration with the configuration system when per-rule configuration is
// implemented. Users will be able to configure: {
// "escape_string_interpolation_operators": { "sensitivity": "conservative" |
// "balanced" | "aggressive" } }

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
