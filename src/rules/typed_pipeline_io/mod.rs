use core::mem;

use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Argument, Block, Call, Expr, Expression, PathMember},
};

use crate::{
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, RuleViolation, Severity},
};

fn has_explicit_type_annotation(signature_span: Option<Span>, ctx: &LintContext) -> bool {
    signature_span.is_some_and(|span| {
        let sig_text = ctx.working_set.get_span_contents(span);
        let sig_str = String::from_utf8_lossy(sig_text);
        sig_str.contains("->")
    })
}

fn has_untyped_pipeline_input(
    signature: &nu_protocol::Signature,
    signature_span: Option<Span>,
    ctx: &LintContext,
) -> bool {
    !has_explicit_type_annotation(signature_span, ctx)
        && signature
            .input_output_types
            .iter()
            .all(|(input_type, _)| matches!(input_type, nu_protocol::Type::Any))
}

fn has_untyped_pipeline_output(
    signature: &nu_protocol::Signature,
    signature_span: Option<Span>,
    ctx: &LintContext,
) -> bool {
    !has_explicit_type_annotation(signature_span, ctx)
        && signature
            .input_output_types
            .iter()
            .all(|(_, output_type)| matches!(output_type, nu_protocol::Type::Any))
}

fn find_signature_span(call: &Call, _ctx: &LintContext) -> Option<Span> {
    let sig_arg = call.get_positional_arg(1)?;
    Some(sig_arg.span)
}

fn create_violations_for_untyped_io(
    func_name: &str,
    name_span: Span,
    uses_in: bool,
    needs_input_type: bool,
    needs_output_type: bool,
    fix: &Fix,
) -> Vec<RuleViolation> {
    let input_violation = needs_input_type.then(|| {
        RuleViolation::new_dynamic(
            "typed_pipeline_io",
            format!(
                "Custom command '{func_name}' uses pipeline input ($in) but lacks input type \
                 annotation"
            ),
            name_span,
        )
        .with_suggestion_static(
            "Add pipeline input type annotation (e.g., `: string -> any` or `: list<int> -> any`)",
        )
        .with_fix(fix.clone())
    });

    let output_violation = needs_output_type.then(|| {
        let suggestion = if uses_in {
            "Add pipeline output type annotation (e.g., `: any -> string` or `: list<int> -> \
             table`)"
        } else {
            "Add pipeline output type annotation (e.g., `: nothing -> string` or `: nothing -> \
             list<int>`)"
        };
        RuleViolation::new_dynamic(
            "typed_pipeline_io",
            format!(
                "Custom command '{func_name}' produces output but lacks output type annotation"
            ),
            name_span,
        )
        .with_suggestion_static(suggestion)
        .with_fix(fix.clone())
    });

    input_violation
        .into_iter()
        .chain(output_violation)
        .collect()
}

#[allow(clippy::excessive_nesting, reason = "Simple nested pattern matching")]
fn infer_output_type(block_id: BlockId, ctx: &LintContext) -> String {
    let block = ctx.working_set.get_block(block_id);

    log::debug!("Inferring output type for block: {block_id:?}");
    log::debug!("Block has {} pipelines", block.pipelines.len());

    // Check if the last pipeline element is a command that produces a list
    if let Some(last_pipeline) = block.pipelines.last()
        && let Some(last_element) = last_pipeline.elements.last()
    {
        log::debug!(
            "Last element expr type: {:?}",
            mem::discriminant(&last_element.expr.expr)
        );

        // Unwrap Collect if present
        let expr = match &last_element.expr.expr {
            Expr::Collect(_, inner) => &inner.expr,
            other => other,
        };

        if let Expr::Subexpression(block_id) | Expr::Block(block_id) = expr {
            // Check the inner block's last pipeline element
            let inner_block = ctx.working_set.get_block(*block_id);
            if let Some(inner_pipeline) = inner_block.pipelines.last()
                && let Some(inner_element) = inner_pipeline.elements.last()
                && let Expr::Call(call) = &inner_element.expr.expr
            {
                let cmd_name = call.get_call_name(ctx);
                log::debug!("Last command is: {cmd_name}");
                match cmd_name.as_str() {
                    "each" | "where" | "filter" | "map" => {
                        log::debug!("Inferred output: list<any>");
                        return "list<any>".to_string();
                    }
                    _ => {}
                }
            }
        } else if let Expr::Call(call) = expr {
            let cmd_name = call.get_call_name(ctx);
            log::debug!("Last command is: {cmd_name}");
            match cmd_name.as_str() {
                "each" | "where" | "filter" | "map" => {
                    log::debug!("Inferred output: list<any>");
                    return "list<any>".to_string();
                }
                _ => {}
            }
        }
    }

    let last_expression = block.output_type();
    log::debug!("Default output type: {last_expression}");
    last_expression.to_string()
}

fn infer_input_type(block_id: BlockId, ctx: &LintContext) -> String {
    log::debug!("Inferring input type for block: {block_id:?}");
    let block = ctx.working_set.get_block(block_id);
    log::debug!("Block span: {:?}", block.span);
    log::debug!("Block pipelines: {}", block.pipelines.len());

    // Find if $in is used in the block by searching for variables with declaration_span (0,0)
    let in_var = find_pipeline_input_in_block(block, ctx);
    log::debug!("$in variable: {in_var:?}");

    if in_var.is_none() {
        return "any".to_string();
    }

    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            if let Some(input_type) = infer_input_from_expression(&element.expr, in_var, ctx) {
                return input_type;
            }
        }
    }

    "any".to_string()
}

fn find_pipeline_input_in_block(block: &Block, ctx: &LintContext) -> Option<VarId> {
    // Search for a variable with declaration_span (0,0) which indicates $in
    log::debug!(
        "find_pipeline_input_in_block: searching {} pipelines",
        block.pipelines.len()
    );
    for (i, pipeline) in block.pipelines.iter().enumerate() {
        log::debug!("  Pipeline {}: {} elements", i, pipeline.elements.len());
        for (j, element) in pipeline.elements.iter().enumerate() {
            log::debug!(
                "    Element {}: span={:?}, ty={:?}",
                j,
                element.expr.span,
                element.expr.ty
            );
            if let Some(var_id) = find_pipeline_input_in_expr(&element.expr, ctx) {
                return Some(var_id);
            }
        }
    }
    None
}

#[allow(
    clippy::uninlined_format_args,
    reason = "Debug logging with many format calls"
)]
fn find_pipeline_input_in_expr(expr: &Expression, ctx: &LintContext) -> Option<VarId> {
    use Expr;
    use std::mem::discriminant;

    log::debug!("find_pipeline_input_in_expr: checking expr");

    match &expr.expr {
        Expr::Var(var_id) => {
            let var = ctx.working_set.get_variable(*var_id);
            log::debug!(
                "  Var: var_id={:?}, decl_span=({}, {}), expr_span=({}, {}), ty={:?}",
                var_id,
                var.declaration_span.start,
                var.declaration_span.end,
                expr.span.start,
                expr.span.end,
                var.ty
            );
            // $in is represented with declaration_span where start == end (pointing to usage location)
            // OR with (0,0) if it's implicit
            if (var.declaration_span.start == 0 && var.declaration_span.end == 0)
                || (var.declaration_span.start == var.declaration_span.end
                    && var.declaration_span.start > 0)
            {
                log::debug!("    -> Found $in variable!");
                return Some(*var_id);
            }
            None
        }
        Expr::FullCellPath(cell_path) => find_pipeline_input_in_expr(&cell_path.head, ctx),
        Expr::Call(call) => {
            log::debug!("  Call: checking {} arguments", call.arguments.len());
            for arg in &call.arguments {
                let expr = match arg {
                    Argument::Positional(e)
                    | Argument::Unknown(e)
                    | Argument::Named((_, _, Some(e)))
                    | Argument::Spread(e) => Some(e),
                    Argument::Named(_) => None,
                };
                if let Some(e) = expr
                    && let Some(var_id) = find_pipeline_input_in_expr(e, ctx)
                {
                    return Some(var_id);
                }
            }
            None
        }
        Expr::BinaryOp(lhs, _, rhs) => {
            find_pipeline_input_in_expr(lhs, ctx).or_else(|| find_pipeline_input_in_expr(rhs, ctx))
        }
        Expr::UnaryNot(e) => find_pipeline_input_in_expr(e, ctx),
        Expr::Collect(_, inner) => {
            log::debug!("  Collect: recursing into inner expression");
            find_pipeline_input_in_expr(inner, ctx)
        }
        Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
            log::debug!("  Subexpression/Block/Closure: block_id={:?}", block_id);
            let block = ctx.working_set.get_block(*block_id);
            find_pipeline_input_in_block(block, ctx)
        }
        Expr::StringInterpolation(items) => {
            for item in items {
                if let Some(var_id) = find_pipeline_input_in_expr(item, ctx) {
                    return Some(var_id);
                }
            }
            None
        }
        other => {
            log::debug!("  Other expression type: {:?}", discriminant(other));
            None
        }
    }
}

fn infer_input_from_expression(
    expr: &Expression,
    in_var: Option<nu_protocol::VarId>,
    ctx: &LintContext,
) -> Option<String> {
    let in_var_id = in_var?;
    log::debug!("Inferring input type from expression type: {:?}", expr.expr);
    match &expr.expr {
        Expr::FullCellPath(cell_path) => {
            log::debug!("Branch: FullCellPath");
            if let Expr::Var(var_id) = &cell_path.head.expr
                && *var_id == in_var_id
                && !cell_path.tail.is_empty()
            {
                log::debug!("$in is used in cell path");
                if cell_path
                    .tail
                    .iter()
                    .any(|member| matches!(member, PathMember::String { .. }))
                {
                    log::debug!("Inferred: record (has string member)");
                    return Some("record".to_string());
                }
                log::debug!("Inferred: list (no string member)");
                return Some("list".to_string());
            }
            log::debug!("FullCellPath: no match, returning None");
            None
        }
        Expr::Call(call) => {
            log::debug!("Branch: Call");
            let cmd_name = call.get_call_name(ctx);
            log::debug!("Call command name: {cmd_name}");

            // Check if this is a command that indicates the input type
            // Commands like `each`, `where`, `filter` indicate list input
            // Commands like `lines`, `split row` indicate string input
            match cmd_name.as_str() {
                "each" | "where" | "filter" | "reduce" | "map" => {
                    log::debug!("Inferred: list (from {cmd_name} command)");
                    Some("list<any>".to_string())
                }
                "lines" | "split row" => {
                    log::debug!("Inferred: string (from {cmd_name} command)");
                    Some("string".to_string())
                }
                _ => {
                    log::debug!("Call: unknown command {cmd_name}, returning None");
                    None
                }
            }
        }
        Expr::Collect(_, inner) => {
            log::debug!("Branch: Collect, recursing into inner expression");
            let result = infer_input_from_expression(inner, in_var, ctx);
            log::debug!("Collect result: {result:?}");
            result
        }
        Expr::BinaryOp(left, _, right) => {
            log::debug!("Branch: BinaryOp");
            infer_input_from_expression(left, in_var, ctx)
                .or_else(|| infer_input_from_expression(right, in_var, ctx))
        }
        Expr::UnaryNot(inner) => {
            log::debug!("Branch: UnaryNot");
            infer_input_from_expression(inner, in_var, ctx)
        }
        Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
            log::debug!("Branch: Subexpression/Block/Closure, block_id: {block_id:?}");
            // Traverse into the block to find expressions that use $in
            let block = ctx.working_set.get_block(*block_id);
            log::debug!("Block has {} pipelines", block.pipelines.len());
            let result = block
                .pipelines
                .iter()
                .flat_map(|pipeline| {
                    log::debug!("Pipeline has {} elements", pipeline.elements.len());
                    &pipeline.elements
                })
                .find_map(|element| {
                    log::debug!(
                        "Checking pipeline element expression type: {:?}",
                        element.expr.expr
                    );
                    infer_input_from_expression(&element.expr, in_var, ctx)
                });
            log::debug!("Subexpression/Block/Closure result: {result:?}");
            result
        }
        other => {
            log::debug!("Branch: Other ({other:?}), returning None");
            None
        }
    }
}

fn generate_typed_signature(
    signature: &nu_protocol::Signature,
    ctx: &LintContext,
    block_id: BlockId,
    uses_in: bool,
    needs_input_type: bool,
    needs_output_type: bool,
) -> String {
    log::debug!("Generating typed signature");
    let has_params = signature.required_positional.is_empty()
        && signature.optional_positional.is_empty()
        && signature.rest_positional.is_none()
        && signature.named.is_empty();
    log::debug!("Has parameters: {has_params}");
    let params_text = if has_params {
        String::new()
    } else {
        extract_parameters_text(signature)
    };

    let input_type = if uses_in || needs_input_type {
        infer_input_type(block_id, ctx)
    } else {
        "nothing".to_string()
    };

    let output_type = if needs_output_type {
        infer_output_type(block_id, ctx)
    } else {
        "any".to_string()
    };

    if needs_input_type || needs_output_type {
        format!("[{params_text}]: {input_type} -> {output_type}")
    } else {
        format!("[{params_text}]")
    }
}

fn extract_parameters_text(signature: &nu_protocol::Signature) -> String {
    let required = signature
        .required_positional
        .iter()
        .map(|param| match param.shape {
            nu_protocol::SyntaxShape::Any => param.name.clone(),
            _ => format!("{}: {}", param.name, shape_to_string(&param.shape)),
        });

    let optional = signature
        .optional_positional
        .iter()
        .map(|param| match param.shape {
            nu_protocol::SyntaxShape::Any => format!("{}?", param.name),
            _ => format!("{}?: {}", param.name, shape_to_string(&param.shape)),
        });

    let rest = signature
        .rest_positional
        .iter()
        .map(|rest| match rest.shape {
            nu_protocol::SyntaxShape::Any => format!("...{}", rest.name),
            _ => format!("...{}: {}", rest.name, shape_to_string(&rest.shape)),
        });

    let flags = signature
        .named
        .iter()
        .filter(|flag| flag.long != "help")
        .map(|flag| match (&flag.short, &flag.arg) {
            (Some(short), Some(arg_shape)) => {
                format!(
                    "--{} (-{}): {}",
                    flag.long,
                    short,
                    shape_to_string(arg_shape)
                )
            }
            (Some(short), None) => format!("--{} (-{})", flag.long, short),
            (None, Some(arg_shape)) => {
                format!("--{}: {}", flag.long, shape_to_string(arg_shape))
            }
            (None, None) => format!("--{}", flag.long),
        });

    required
        .chain(optional)
        .chain(rest)
        .chain(flags)
        .collect::<Vec<_>>()
        .join(", ")
}

fn shape_to_string(shape: &nu_protocol::SyntaxShape) -> String {
    use nu_protocol::SyntaxShape;

    match shape {
        SyntaxShape::Int => "int".to_string(),
        SyntaxShape::String => "string".to_string(),
        SyntaxShape::Float => "float".to_string(),
        SyntaxShape::Boolean => "bool".to_string(),
        SyntaxShape::List(inner) => format!("list<{}>", shape_to_string(inner)),
        SyntaxShape::Table(cols) if cols.is_empty() => "table".to_string(),
        SyntaxShape::Table(cols) => {
            let col_names = cols
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("table<{col_names}>")
        }
        SyntaxShape::Record(_) => "record".to_string(),
        SyntaxShape::Filepath => "path".to_string(),
        SyntaxShape::Directory => "directory".to_string(),
        SyntaxShape::GlobPattern => "glob".to_string(),
        SyntaxShape::Any => "any".to_string(),
        _ => format!("{shape:?}").to_lowercase(),
    }
}

fn check_def_call(call: &Call, ctx: &LintContext) -> Vec<RuleViolation> {
    let Some((block_id, func_name)) = call.extract_function_definition(ctx) else {
        log::debug!(
            "Could not find function definition for call to '{}'",
            call.get_call_name(ctx)
        );
        return vec![];
    };

    let Some((_, name_span)) = call.extract_declaration_name(ctx) else {
        log::debug!("Could not find declaration name span for function '{func_name}'");
        return vec![];
    };

    log::debug!("Checking function definition for '{func_name}'");

    let block = ctx.working_set.get_block(block_id);
    let signature = &block.signature;

    let sig_span = find_signature_span(call, ctx);
    let uses_in = block_id.uses_pipeline_input(ctx);
    log::debug!("Uses pipeline input: {uses_in}");
    let has_untyped_input = has_untyped_pipeline_input(signature, sig_span, ctx);
    log::debug!("Has untyped pipeline input: {has_untyped_input}");
    let has_untyped_output = has_untyped_pipeline_output(signature, sig_span, ctx);
    log::debug!("Has untyped pipeline output: {has_untyped_output}");
    let produces_out = block_id.produces_output(ctx);
    log::debug!("Produces output: {produces_out}");
    let needs_input_type = uses_in && has_untyped_input;
    log::debug!("Needs input type annotation: {needs_input_type}");
    let needs_output_type = produces_out && has_untyped_output;
    log::debug!("Needs output type annotation: {needs_output_type}");
    if !needs_input_type && !needs_output_type {
        return vec![];
    }

    let Some(sig_span) = sig_span else {
        return vec![];
    };

    let new_signature = generate_typed_signature(
        signature,
        ctx,
        block_id,
        uses_in,
        needs_input_type,
        needs_output_type,
    );

    let fix = Fix::new_dynamic(
        format!("Add type annotations: {new_signature}"),
        vec![Replacement::new_dynamic(sig_span, new_signature)],
    );

    create_violations_for_untyped_io(
        &func_name,
        name_span,
        uses_in,
        needs_input_type,
        needs_output_type,
        &fix,
    )
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) => check_def_call(call, ctx),
        _ => vec![],
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "typed_pipeline_io",
        RuleCategory::TypeSafety,
        Severity::Warning,
        "Custom commands that use pipeline input or produce output should have type annotations",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
