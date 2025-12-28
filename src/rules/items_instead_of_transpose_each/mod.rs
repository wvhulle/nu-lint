use nu_protocol::ast::{Argument, Call, Expr, Expression, Pipeline, Traverse};

use crate::{
    Fix, LintLevel, Replacement,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct TransposeEachPattern {
    each_call: Call,
    col1: String,
    col2: String,
    combined_span: nu_protocol::Span,
    transpose_span: nu_protocol::Span,
    each_span: nu_protocol::Span,
    closure_span: nu_protocol::Span,
}

fn extract_transpose_column_names(call: &Call) -> Option<(String, String)> {
    if call.arguments.len() != 2 {
        return None;
    }

    let first_arg = call.arguments.first()?;
    let second_arg = call.arguments.get(1)?;

    let first_name = match first_arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => {
            if let Expr::String(s) = &expr.expr {
                s.clone()
            } else {
                return None;
            }
        }
        _ => return None,
    };

    let second_name = match second_arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => {
            if let Expr::String(s) = &expr.expr {
                s.clone()
            } else {
                return None;
            }
        }
        _ => return None,
    };

    Some((first_name, second_name))
}

fn closure_only_uses_fields(
    block_id: nu_protocol::BlockId,
    closure_var_id: nu_protocol::VarId,
    field1: &str,
    field2: &str,
    context: &LintContext,
) -> bool {
    let block = context.working_set.get_block(block_id);

    let mut field_accesses = Vec::new();

    block.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::FullCellPath(cell_path) = &expr.expr {
                if let Expr::Var(var_id) = &cell_path.head.expr
                    && *var_id == closure_var_id
                {
                    vec![expr.clone()]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        },
        &mut field_accesses,
    );

    log::debug!("Found {} field accesses", field_accesses.len());

    if field_accesses.is_empty() {
        log::debug!("No field accesses found");
        return false;
    }

    let all_valid = field_accesses.iter().all(|expr| {
        if let Expr::FullCellPath(cell_path) = &expr.expr
            && cell_path.tail.len() == 1
        {
            let field_name = context.get_span_text(cell_path.tail[0].span());
            log::debug!("Field access: {field_name} (expecting {field1} or {field2})");
            field_name == field1 || field_name == field2
        } else {
            log::debug!("Invalid field access pattern");
            false
        }
    });

    log::debug!("All field accesses valid: {all_valid}");
    all_valid
}

fn detect_pattern_in_pipeline(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Vec<TransposeEachPattern> {
    let mut patterns = Vec::new();

    log::debug!(
        "Checking pipeline with {} elements",
        pipeline.elements.len()
    );

    for i in 0..pipeline.elements.len().saturating_sub(1) {
        let transpose_elem = &pipeline.elements[i];
        let each_elem = &pipeline.elements[i + 1];

        let Expr::Call(transpose_call) = &transpose_elem.expr.expr else {
            continue;
        };

        log::debug!("Found call at index {i}");

        if !transpose_call.is_call_to_command("transpose", context) {
            log::debug!("Not transpose command");
            continue;
        }

        log::debug!("Found transpose command");

        let Some((col1, col2)) = extract_transpose_column_names(transpose_call) else {
            log::debug!("Failed to extract 2 column names");
            continue;
        };

        log::debug!("Column names: {col1}, {col2}");

        let Expr::Call(each_call) = &each_elem.expr.expr else {
            log::debug!("Next element is not a call");
            continue;
        };

        if !each_call.is_call_to_command("each", context) {
            log::debug!("Next call is not each");
            continue;
        }

        log::debug!("Found each call");

        let Some(Argument::Positional(closure_arg) | Argument::Unknown(closure_arg)) =
            each_call.arguments.first()
        else {
            log::debug!("Each doesn't have positional argument");
            continue;
        };

        let Expr::Closure(block_id) = &closure_arg.expr else {
            log::debug!("Argument is not a closure");
            continue;
        };

        log::debug!("Closure found");

        let block = context.working_set.get_block(*block_id);

        if block.signature.required_positional.len() != 1 {
            log::debug!(
                "Closure has {} parameters",
                block.signature.required_positional.len()
            );
            continue;
        }

        log::debug!("Closure has 1 parameter");

        let param = &block.signature.required_positional[0];
        let Some(closure_var_id) = param.var_id else {
            log::debug!("Parameter doesn't have var_id");
            continue;
        };

        log::debug!("Checking field usage");

        if !closure_only_uses_fields(*block_id, closure_var_id, &col1, &col2, context) {
            log::debug!("Closure doesn't only use the specified fields");
            continue;
        }

        log::debug!("Pattern matched!");

        let combined_span =
            nu_protocol::Span::new(transpose_call.head.start, each_elem.expr.span.end);

        patterns.push(TransposeEachPattern {
            each_call: *each_call.clone(),
            col1,
            col2,
            combined_span,
            transpose_span: transpose_call.head,
            each_span: each_call.head,
            closure_span: closure_arg.span,
        });
    }

    patterns
}

fn collect_patterns_from_block(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
) -> Vec<TransposeEachPattern> {
    let block = context.working_set.get_block(block_id);
    let mut patterns = Vec::new();

    for pipeline in &block.pipelines {
        patterns.extend(detect_pattern_in_pipeline(pipeline, context));
    }

    patterns
}

fn detect_pattern(expr: &Expression, context: &LintContext) -> Vec<TransposeEachPattern> {
    match &expr.expr {
        Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
            collect_patterns_from_block(*block_id, context)
        }
        _ => vec![],
    }
}

struct ItemsInsteadOfTransposeEach;

impl DetectFix for ItemsInsteadOfTransposeEach {
    type FixInput<'a> = TransposeEachPattern;

    fn id(&self) -> &'static str {
        "items_instead_of_transpose_each"
    }

    fn explanation(&self) -> &'static str {
        "Use 'items' instead of 'transpose | each' when iterating over record entries"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/items.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let context: &LintContext = context;
        let mut patterns = Vec::new();

        log::debug!(
            "Checking {} top-level pipelines",
            context.ast.pipelines.len()
        );

        for pipeline in &context.ast.pipelines {
            patterns.extend(detect_pattern_in_pipeline(pipeline, context));
        }

        log::debug!("Found {} patterns in top-level pipelines", patterns.len());

        context.ast.flat_map(
            context.working_set,
            &|expr| detect_pattern(expr, context),
            &mut patterns,
        );

        log::debug!("Found {} total patterns after traversal", patterns.len());

        patterns
            .into_iter()
            .map(|pattern| {
                let violation = Detection::from_global_span(
                    "Use 'items' instead of 'transpose | each' when iterating over record entries",
                    pattern.combined_span,
                )
                .with_primary_label("transpose | each pattern")
                .with_extra_label("converts record to table", pattern.transpose_span)
                .with_extra_label("iterates over rows", pattern.each_span)
                .with_extra_label(
                    format!("accesses ${} and ${}", pattern.col1, pattern.col2),
                    pattern.closure_span,
                )
                .with_help(
                    "The 'items' command directly iterates over key-value pairs without needing \
                     transpose",
                );
                (violation, pattern)
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, pattern: &Self::FixInput<'_>) -> Option<Fix> {
        let pattern: &TransposeEachPattern = pattern;
        let closure_arg = pattern.each_call.arguments.first()?;
        let (Argument::Positional(closure_expr) | Argument::Unknown(closure_expr)) = closure_arg
        else {
            return None;
        };

        let Expr::Closure(block_id) = &closure_expr.expr else {
            return None;
        };

        let block = context.working_set.get_block(*block_id);
        let param = &block.signature.required_positional[0];

        let closure_body_text = context.get_span_text(block.span?);
        let mut closure_body_trimmed = closure_body_text
            .trim()
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
            .map_or(closure_body_text.trim(), |s| s.trim());

        if let Some(pipe_pos) = closure_body_trimmed.find('|')
            && let Some(second_pipe) = closure_body_trimmed[pipe_pos + 1..].find('|')
        {
            closure_body_trimmed = closure_body_trimmed[pipe_pos + second_pipe + 2..].trim();
        }

        let param_name = &param.name;
        let new_body = closure_body_trimmed
            .replace(
                &format!("${param_name}.{}", pattern.col1),
                &format!("${}", pattern.col1),
            )
            .replace(
                &format!("${param_name}.{}", pattern.col2),
                &format!("${}", pattern.col2),
            );

        let fix_text = format!("items {{|{}, {}| {new_body} }}", pattern.col1, pattern.col2);

        Some(Fix::with_explanation(
            "Replace 'transpose ... | each' with 'items' for cleaner iteration over record entries",
            vec![Replacement::new(pattern.combined_span, fix_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &ItemsInsteadOfTransposeEach;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
