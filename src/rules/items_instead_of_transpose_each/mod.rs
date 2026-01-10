use nu_protocol::{
    Span,
    ast::{Argument, Call, Expr, Expression, Pipeline, Traverse},
};

use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, pipeline::PipelineExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

/// Represents a cell path access like `$row.col1` that needs to be replaced
/// with `$col1`
struct CellPathReplacement {
    /// The full span of `$row.col1`
    span: Span,
    /// The field name (e.g., "col1")
    field: String,
}

struct TransposeEachPattern {
    each_call: Call,
    col1: String,
    col2: String,
    combined_span: nu_protocol::Span,
    transpose_span: nu_protocol::Span,
    each_span: nu_protocol::Span,
    closure_span: nu_protocol::Span,
    /// Cell path accesses that need replacement
    cell_path_replacements: Vec<CellPathReplacement>,
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
) -> Option<Vec<CellPathReplacement>> {
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
        return None;
    }

    let mut replacements = Vec::new();

    for expr in &field_accesses {
        if let Expr::FullCellPath(cell_path) = &expr.expr
            && cell_path.tail.len() == 1
        {
            let field_name = context.plain_text(cell_path.tail[0].span());
            log::debug!("Field access: {field_name} (expecting {field1} or {field2})");
            if field_name == field1 || field_name == field2 {
                replacements.push(CellPathReplacement {
                    span: expr.span,
                    field: field_name.to_string(),
                });
            } else {
                log::debug!("Invalid field name");
                return None;
            }
        } else {
            log::debug!("Invalid field access pattern");
            return None;
        }
    }

    log::debug!("All {} field accesses valid", replacements.len());
    Some(replacements)
}

fn detect_pattern_in_pipeline(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Vec<TransposeEachPattern> {
    log::debug!(
        "Checking pipeline with {} elements",
        pipeline.elements.len()
    );

    pipeline
        .find_command_pairs(
            context,
            |call, ctx| call.is_call_to_command("transpose", ctx),
            |call, ctx| call.is_call_to_command("each", ctx),
        )
        .into_iter()
        .filter_map(|pair| {
            log::debug!("Found transpose | each pair");

            let Some((col1, col2)) = extract_transpose_column_names(pair.first) else {
                log::debug!("Failed to extract 2 column names");
                return None;
            };

            log::debug!("Column names: {col1}, {col2}");

            let Some(Argument::Positional(closure_arg) | Argument::Unknown(closure_arg)) =
                pair.second.arguments.first()
            else {
                log::debug!("Each doesn't have positional argument");
                return None;
            };

            let Expr::Closure(block_id) = &closure_arg.expr else {
                log::debug!("Argument is not a closure");
                return None;
            };

            log::debug!("Closure found");

            let block = context.working_set.get_block(*block_id);

            if block.signature.required_positional.len() != 1 {
                log::debug!(
                    "Closure has {} parameters",
                    block.signature.required_positional.len()
                );
                return None;
            }

            log::debug!("Closure has 1 parameter");

            let param = &block.signature.required_positional[0];
            let Some(closure_var_id) = param.var_id else {
                log::debug!("Parameter doesn't have var_id");
                return None;
            };

            log::debug!("Checking field usage");

            let Some(cell_path_replacements) =
                closure_only_uses_fields(*block_id, closure_var_id, &col1, &col2, context)
            else {
                log::debug!("Closure doesn't only use the specified fields");
                return None;
            };

            log::debug!("Pattern matched!");

            Some(TransposeEachPattern {
                each_call: pair.second.clone(),
                col1,
                col2,
                combined_span: pair.span,
                transpose_span: pair.first.head,
                each_span: pair.second.head,
                closure_span: closure_arg.span,
                cell_path_replacements,
            })
        })
        .collect()
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
        "transpose_items"
    }

    fn short_description(&self) -> &'static str {
        "Use 'items' instead of 'transpose | each' when iterating over record entries"
    }

    fn source_link(&self) -> Option<&'static str> {
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

        let mut replacements = Vec::new();

        // 1. Replace `transpose col1 col2 | each` with `items`
        // The span from transpose head to just before the closure
        let transpose_each_span = Span::new(pattern.transpose_span.start, closure_expr.span.start);
        replacements.push(Replacement::new(transpose_each_span, "items ".to_string()));

        // 2. Replace the closure parameter declaration `|row|` with `|col1, col2|`
        // Find the parameter span in the closure - it's between the first { and the
        // body
        let closure_text = context.plain_text(closure_expr.span);

        // Find parameter declaration span: from after `{` to before body
        // The parameter is at param.name with its span
        if param.var_id.is_some() {
            // Find where the parameter name is declared in the closure signature
            // The signature spans from `{|` to `|` before body
            let _body_span = block.span?;

            // The param declaration is between the opening `{|` and the closing `|` before
            // body We need to find the span of `|param_name|` and replace with
            // `|col1, col2|`
            let closure_start = closure_expr.span.start;

            // The parameter list span is from closure_start to body_start (includes {| and
            // |}) But we only want to replace the parameter name itself
            // Find `|param_name|` pattern in the closure header
            if let Some(open_pipe_offset) = closure_text.find('|') {
                let after_open_pipe = &closure_text[open_pipe_offset + 1..];
                if let Some(close_pipe_offset) = after_open_pipe.find('|') {
                    // Span of the parameter list (between the pipes)
                    let param_list_start = closure_start + open_pipe_offset + 1;
                    let param_list_end = closure_start + open_pipe_offset + 1 + close_pipe_offset;
                    let param_list_span = Span::new(param_list_start, param_list_end);

                    replacements.push(Replacement::new(
                        param_list_span,
                        format!("{}, {}", pattern.col1, pattern.col2),
                    ));
                }
            }
        }

        // 3. Replace each cell path access `$row.col1` with `$col1`, `$row.col2` with
        //    `$col2`
        for cell_path_replacement in &pattern.cell_path_replacements {
            replacements.push(Replacement::new(
                cell_path_replacement.span,
                format!("${}", cell_path_replacement.field),
            ));
        }

        Some(Fix::with_explanation(
            "Replace 'transpose ... | each' with 'items' for cleaner iteration over record entries",
            replacements,
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
