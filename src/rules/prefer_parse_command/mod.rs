use nu_protocol::{
    Span, VarId,
    ast::{Argument, Block, Call, Expr, Expression, Pipeline, PipelineElement},
};

use crate::{
    Fix, Replacement,
    ast::{block::BlockExt, call::CallExt, pipeline::PipelineExt, span::SpanExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

const REGEX_SPECIAL_CHARS: &[char] = &[
    '\\', '.', '+', '*', '?', '(', ')', '[', ']', '{', '}', '|', '^', '$',
];

fn is_split_row_call(call: &Call, context: &LintContext) -> bool {
    call.is_call_to_command("split row", context)
}

fn is_split_call(call: &Call, context: &LintContext) -> bool {
    matches!(call.get_call_name(context).as_str(), "split row" | "split")
}

fn extract_delimiter_from_split_call(call: &Call, context: &LintContext) -> Option<String> {
    if !is_split_call(call, context) {
        return None;
    }
    let arg = call.get_first_positional_arg()?;
    let text = arg.span.source_code(context);
    match &arg.expr {
        Expr::String(s) | Expr::RawString(s) => Some(s.clone()),
        _ => {
            let trimmed = text.trim();
            let is_quoted = (trimmed.starts_with('"') && trimmed.ends_with('"'))
                || (trimmed.starts_with('\'') && trimmed.ends_with('\''));
            is_quoted.then(|| trimmed[1..trimmed.len() - 1].to_string())
        }
    }
}

fn needs_regex_for_delimiter(delimiter: &str) -> bool {
    delimiter.chars().any(|c| REGEX_SPECIAL_CHARS.contains(&c))
}

fn escape_regex_delimiter(delimiter: &str) -> String {
    delimiter.chars().fold(
        String::with_capacity(delimiter.len() * 2),
        |mut escaped, c| {
            if REGEX_SPECIAL_CHARS.contains(&c) {
                escaped.push('\\');
            }
            escaped.push(c);
            escaped
        },
    )
}

fn generate_parse_pattern(delimiter: &str, num_fields: usize) -> (String, bool) {
    let needs_regex = needs_regex_for_delimiter(delimiter);

    if needs_regex {
        let escaped = escape_regex_delimiter(delimiter);
        let pattern = (0..num_fields)
            .map(|i| format!("(?P<field{i}>.*)"))
            .collect::<Vec<_>>()
            .join(&escaped);
        (pattern, true)
    } else {
        let pattern = (0..num_fields)
            .map(|i| format!("{{field{i}}}"))
            .collect::<Vec<_>>()
            .join(delimiter);
        (pattern, false)
    }
}

fn generate_parse_replacement(delimiter: &str, indexed_fields: &[usize]) -> String {
    let max_field = indexed_fields.iter().copied().max().unwrap_or(0);
    let num_fields = max_field + 2;
    let (pattern, needs_regex) = generate_parse_pattern(delimiter, num_fields);

    if needs_regex {
        format!("parse --regex '{pattern}'")
    } else {
        format!("parse \"{pattern}\"")
    }
}

fn contains_split_in_expression(expr: &Expression, ctx: &LintContext) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            is_split_call(call, ctx)
                || call.arguments.iter().any(|arg| {
                    matches!(arg,
                        Argument::Positional(e) | Argument::Named((_, _, Some(e)))
                        if contains_split_in_expression(e, ctx)
                    )
                })
        }
        Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => ctx
            .working_set
            .get_block(*id)
            .pipelines
            .iter()
            .flat_map(|p| &p.elements)
            .any(|elem| contains_split_in_expression(&elem.expr, ctx)),
        Expr::FullCellPath(path) => contains_split_in_expression(&path.head, ctx),
        Expr::BinaryOp(left, _, right) => {
            contains_split_in_expression(left, ctx) || contains_split_in_expression(right, ctx)
        }
        Expr::UnaryNot(inner) => contains_split_in_expression(inner, ctx),
        _ => false,
    }
}

fn check_each_with_split(expr: &Expression, ctx: &LintContext) -> Option<Violation> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };
    if !call.is_call_to_command("each", ctx) {
        return None;
    }

    let has_split = call
        .arguments
        .iter()
        .any(|arg| matches!(arg, Argument::Positional(e) if contains_split_in_expression(e, ctx)));

    has_split.then(|| {
        Violation::new(
            "Manual splitting with 'each' and 'split row' - consider using 'parse'",
            call.span(),
        )
        .with_primary_label("manual split pattern")
        .with_help(
            "Use 'parse \"{field0} {field1}\"' for structured text extraction instead of 'each' \
             with 'split row'. For complex delimiters, use 'parse --regex' with named capture \
             groups like '(?P<field0>.*)delimiter(?P<field1>.*)'",
        )
    })
}

fn is_indexed_access_call(call: &Call, context: &LintContext) -> bool {
    matches!(call.get_call_name(context).as_str(), "get" | "skip")
}

fn extract_index_from_call(call: &Call, context: &LintContext) -> Option<usize> {
    call.get_first_positional_arg()
        .and_then(|arg| arg.span.source_code(context).parse().ok())
}

fn check_pipeline_for_split_get(pipeline: &Pipeline, context: &LintContext) -> Option<Violation> {
    if pipeline.elements.len() < 2 {
        return None;
    }

    pipeline.elements.windows(2).find_map(|window| {
        let [current, next] = window else {
            return None;
        };
        let (Expr::Call(split_call), Expr::Call(access_call)) =
            (&current.expr.expr, &next.expr.expr)
        else {
            return None;
        };

        if !is_split_row_call(split_call, context) || !is_indexed_access_call(access_call, context)
        {
            return None;
        }

        let index = extract_index_from_call(access_call, context)?;
        let span = Span::new(current.expr.span.start, next.expr.span.end);

        let delimiter = extract_delimiter_from_split_call(split_call, context);
        let violation = Violation::new(
            "Manual string splitting with indexed access - consider using 'parse'",
            span,
        )
        .with_primary_label("split + index pattern");

        Some(if let Some(delim) = delimiter {
            let replacement = generate_parse_replacement(&delim, &[index]);
            violation
                .with_help(format!(
                    "Use '{replacement}' for structured text extraction. Access fields by name \
                     (e.g., $result.field{index}) instead of index."
                ))
                .with_fix(Fix::with_explanation(
                    format!("Replace 'split row | get/skip' with '{replacement}'"),
                    vec![Replacement::new(span, replacement)],
                ))
        } else {
            violation.with_help(
                "Use 'parse \"{field0} {field1}\"' for structured text extraction. For complex \
                 delimiters containing regex special characters, use 'parse --regex' with named \
                 capture groups like '(?P<field0>.*)delimiter(?P<field1>.*)'",
            )
        })
    })
}

fn extract_split_row_assignment(
    expr: &Expression,
    context: &LintContext,
) -> Option<(VarId, String, Span)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if !call.is_call_to_command("let", context) {
        return None;
    }

    let (var_id, var_name, _var_span) = call.extract_variable_declaration(context)?;
    let value_expr = call.get_positional_arg(1)?;

    log::debug!("Checking let statement for variable: {var_name}");

    let is_split_row_assignment = match &value_expr.expr {
        Expr::Call(value_call) => is_split_row_call(value_call, context),
        Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
            Expr::Call(head_call) => is_split_row_call(head_call, context),
            Expr::Subexpression(block_id) => context
                .working_set
                .get_block(*block_id)
                .contains_call_in_single_pipeline("split row", context),
            _ => false,
        },
        Expr::Subexpression(block_id) | Expr::Block(block_id) => context
            .working_set
            .get_block(*block_id)
            .contains_call_in_single_pipeline("split row", context),
        _ => false,
    };

    is_split_row_assignment.then(|| {
        log::debug!("Variable {var_name} assigned from split row");
        (var_id, var_name, expr.span)
    })
}

fn is_var_used_in_indexed_access(var_id: VarId, call: &Call, context: &LintContext) -> bool {
    if !is_indexed_access_call(call, context) || extract_index_from_call(call, context).is_none() {
        return false;
    }

    call.arguments.iter().any(|arg| {
        matches!(
            arg,
            Argument::Positional(arg_expr)
            | Argument::Unknown(arg_expr)
            if matches!(&arg_expr.expr, Expr::Var(ref_var_id) if *ref_var_id == var_id)
        )
    })
}

fn create_indexed_access_violation(var_name: &str, decl_span: Span) -> Violation {
    Violation::new(
        format!(
            "Variable '{var_name}' from split row with indexed access - consider using 'parse'"
        ),
        decl_span,
    )
    .with_primary_label("split result with indexed access")
    .with_help(
        "Use 'parse' command to extract named fields instead of indexed access. For simple \
         delimiters like space or colon, use 'parse \"{field0} {field1}\"'. For complex \
         delimiters or variable patterns, use 'parse --regex \
         \"(?P<field0>.*)delimiter(?P<field1>.*)\"'",
    )
}

fn check_call_arguments_for_violation(
    call: &Call,
    var_id: VarId,
    var_name: &str,
    decl_span: Span,
    context: &LintContext,
) -> Option<Violation> {
    call.arguments.iter().find_map(|arg| {
        let (Argument::Positional(arg_expr) | Argument::Unknown(arg_expr)) = arg else {
            return None;
        };

        if let Expr::Block(block_id) = &arg_expr.expr {
            let nested_block = context.working_set.get_block(*block_id);
            check_for_indexed_variable_access(var_id, var_name, decl_span, nested_block, context)
        } else {
            None
        }
    })
}

fn check_element_for_indexed_access(
    element: &PipelineElement,
    var_id: VarId,
    var_name: &str,
    decl_span: Span,
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<Violation> {
    match &element.expr.expr {
        Expr::FullCellPath(cp) => {
            if let Expr::Subexpression(block_id) = &cp.head.expr {
                let nested_block = context.working_set.get_block(*block_id);
                return check_for_indexed_variable_access(
                    var_id,
                    var_name,
                    decl_span,
                    nested_block,
                    context,
                );
            }
            None
        }
        Expr::Call(call) => {
            if is_var_used_in_indexed_access(var_id, call, context)
                || (is_indexed_access_call(call, context)
                    && extract_index_from_call(call, context).is_some()
                    && pipeline.variable_is_piped(var_id))
            {
                Some(create_indexed_access_violation(var_name, decl_span))
            } else {
                check_call_arguments_for_violation(call, var_id, var_name, decl_span, context)
            }
        }
        Expr::Block(block_id) => {
            let nested_block = context.working_set.get_block(*block_id);
            check_for_indexed_variable_access(var_id, var_name, decl_span, nested_block, context)
        }
        _ => None,
    }
}

fn check_for_indexed_variable_access(
    var_id: VarId,
    var_name: &str,
    decl_span: Span,
    block: &Block,
    context: &LintContext,
) -> Option<Violation> {
    log::debug!("Checking for indexed access of variable: {var_name}");

    block.pipelines.iter().find_map(|pipeline| {
        // If variable is used in this pipeline and there's an indexed access call,
        // report violation
        if pipeline.variable_is_used(var_id) && pipeline.contains_indexed_access(context) {
            log::debug!("Found indexed access for variable {var_name} in pipeline");
            return Some(create_indexed_access_violation(var_name, decl_span));
        }

        // Recursively check nested expressions
        pipeline.elements.iter().find_map(|element| {
            check_element_for_indexed_access(
                element, var_id, var_name, decl_span, pipeline, context,
            )
        })
    })
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<Violation>) {
    // Check for inline split row | get/skip patterns
    for pipeline in &block.pipelines {
        if let Some(violation) = check_pipeline_for_split_get(pipeline, context) {
            violations.push(violation);
        }
    }

    // Check for split row assignment followed by indexed access
    let split_row_violations = block
        .pipelines
        .iter()
        .enumerate()
        .filter(|(_, pipeline)| pipeline.elements.len() == 1)
        .filter_map(|(i, pipeline)| {
            let element = &pipeline.elements[0];
            extract_split_row_assignment(&element.expr, context)
                .map(|(var_id, var_name, decl_span)| (i, var_id, var_name, decl_span))
        })
        .find_map(|(i, var_id, var_name, decl_span)| {
            log::debug!("Found split row assignment: {var_name}, checking subsequent pipelines");

            block.pipelines[(i + 1)..]
                .iter()
                .find_map(|future_pipeline| {
                    check_for_indexed_variable_access(
                        var_id,
                        &var_name,
                        decl_span,
                        &Block {
                            pipelines: vec![future_pipeline.clone()],
                            ..Default::default()
                        },
                        context,
                    )
                })
        });

    if let Some(violation) = split_row_violations {
        violations.push(violation);
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    check_block(context.ast, context, &mut violations);

    violations.extend(context.collect_rule_violations(|expr, ctx| {
        let mut expr_violations = Vec::new();

        // Check for 'each' with 'split row' pattern
        expr_violations.extend(check_each_with_split(expr, ctx));

        // Check nested blocks/closures for split row | get patterns
        if let Expr::Closure(block_id) | Expr::Block(block_id) = &expr.expr {
            let block = ctx.working_set.get_block(*block_id);
            check_block(block, ctx, &mut expr_violations);
        }

        expr_violations
    }));

    violations
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_parse_over_split",
        "Prefer 'parse' command over manual string splitting patterns",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/parse.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
