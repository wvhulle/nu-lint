use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Call, Expr, Expression, PathMember},
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
        String::from_utf8_lossy(sig_text).contains("->")
    })
}

fn is_untyped<F>(
    signature: &nu_protocol::Signature,
    signature_span: Option<Span>,
    ctx: &LintContext,
    selector: F,
) -> bool
where
    F: Fn(&(nu_protocol::Type, nu_protocol::Type)) -> &nu_protocol::Type,
{
    !has_explicit_type_annotation(signature_span, ctx)
        && signature
            .input_output_types
            .iter()
            .all(|types| matches!(selector(types), nu_protocol::Type::Any))
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
    [
        (
            needs_input_type,
            format!(
                "Custom command '{func_name}' uses pipeline input ($in) but lacks input type \
                 annotation"
            ),
            "Add pipeline input type annotation (e.g., `: string -> any` or `: list<int> -> any`)",
        ),
        (
            needs_output_type,
            format!(
                "Custom command '{func_name}' produces output but lacks output type annotation"
            ),
            if uses_in {
                "Add pipeline output type annotation (e.g., `: any -> string` or `: list<int> -> \
                 table`)"
            } else {
                "Add pipeline output type annotation (e.g., `: nothing -> string` or `: nothing -> \
                 list<int>`)"
            },
        ),
    ]
    .into_iter()
    .filter_map(|(needs, message, suggestion)| {
        needs.then(|| {
            RuleViolation::new_dynamic("typed_pipeline_io", message, name_span)
                .with_suggestion_static(suggestion)
                .with_fix(fix.clone())
        })
    })
    .collect()
}

const fn is_filepath_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Filepath(..) | Expr::GlobPattern(..))
}

fn check_filepath_output(expr: &Expr) -> Option<&'static str> {
    // Check ExternalCall head
    if let Expr::ExternalCall(head, _) = expr
        && is_filepath_expr(&head.expr)
    {
        return Some("path");
    }

    // Check direct expression
    if is_filepath_expr(expr) {
        return Some("path");
    }

    // Check inside Collect
    if let Expr::Collect(_, inner) = expr
        && is_filepath_expr(&inner.expr)
    {
        return Some("path");
    }

    None
}

fn infer_output_type(block_id: BlockId, ctx: &LintContext) -> String {
    let block = ctx.working_set.get_block(block_id);

    block
        .pipelines
        .last()
        .and_then(|pipeline| pipeline.elements.last())
        .and_then(|last_element| {
            let expr = &last_element.expr.expr;

            // Check for filepath expressions first
            if let Some(path_type) = check_filepath_output(expr) {
                return Some(path_type.to_string());
            }

            // Unwrap Collect for command checking
            let expr = match expr {
                Expr::Collect(_, inner) => &inner.expr,
                other => other,
            };

            // Check for command-based type inference
            match expr {
                Expr::Subexpression(block_id) | Expr::Block(block_id) => {
                    ctx.working_set
                        .get_block(*block_id)
                        .pipelines
                        .last()
                        .and_then(|inner_pipeline| inner_pipeline.elements.last())
                        .and_then(|inner_element| match &inner_element.expr.expr {
                            Expr::Call(call) => infer_command_output_type(&call.get_call_name(ctx))
                                .map(String::from),
                            _ => None,
                        })
                }
                Expr::Call(call) => {
                    infer_command_output_type(&call.get_call_name(ctx)).map(String::from)
                }
                _ => None,
            }
        })
        .unwrap_or_else(|| block.output_type().to_string())
}

fn infer_command_output_type(cmd_name: &str) -> Option<&'static str> {
    match cmd_name {
        "each" | "where" | "filter" | "map" => Some("list<any>"),
        "length" => Some("int"),
        _ => None,
    }
}

fn infer_input_type(block_id: BlockId, ctx: &LintContext) -> String {
    let block = ctx.working_set.get_block(block_id);
    let Some(in_var) = block_id.find_pipeline_input_variable(ctx) else {
        return "any".to_string();
    };

    block
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .find_map(|element| infer_input_from_expression(&element.expr, Some(in_var), ctx))
        .map_or_else(|| "any".to_string(), String::from)
}

fn infer_input_from_expression(
    expr: &Expression,
    in_var: Option<VarId>,
    ctx: &LintContext,
) -> Option<&'static str> {
    let in_var_id = in_var?;

    match &expr.expr {
        Expr::FullCellPath(cell_path) if matches!(&cell_path.head.expr, Expr::Var(var_id) if *var_id == in_var_id) => {
            if !cell_path.tail.is_empty()
                && cell_path
                    .tail
                    .iter()
                    .any(|member| matches!(member, PathMember::String { .. }))
            {
                Some("record")
            } else if !cell_path.tail.is_empty() {
                Some("list")
            } else {
                None
            }
        }
        Expr::Call(call) => match call.get_call_name(ctx).as_str() {
            "each" | "where" | "filter" | "reduce" | "map" | "length" => Some("list<any>"),
            "lines" | "split row" => Some("string"),
            _ => None,
        },
        Expr::Collect(_, inner) | Expr::UnaryNot(inner) => {
            infer_input_from_expression(inner, in_var, ctx)
        }
        Expr::BinaryOp(left, _, right) => infer_input_from_expression(left, in_var, ctx)
            .or_else(|| infer_input_from_expression(right, in_var, ctx)),
        Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
            let block = ctx.working_set.get_block(*block_id);
            block
                .pipelines
                .iter()
                .flat_map(|pipeline| &pipeline.elements)
                .find_map(|element| infer_input_from_expression(&element.expr, in_var, ctx))
        }
        _ => None,
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
    let has_no_params = signature.required_positional.is_empty()
        && signature.optional_positional.is_empty()
        && signature.rest_positional.is_none()
        && signature.named.is_empty();

    let params_text = if has_no_params {
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

    match (needs_input_type, needs_output_type) {
        (false, false) => format!("[{params_text}]"),
        _ => format!("[{params_text}]: {input_type} -> {output_type}"),
    }
}

fn extract_parameters_text(signature: &nu_protocol::Signature) -> String {
    let required = signature
        .required_positional
        .iter()
        .map(|param| format_positional(&param.name, &param.shape, false, false));

    let optional = signature
        .optional_positional
        .iter()
        .map(|param| format_positional(&param.name, &param.shape, true, false));

    let rest = signature
        .rest_positional
        .iter()
        .map(|rest| format_positional(&rest.name, &rest.shape, false, true));

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

fn format_positional(
    name: &str,
    shape: &nu_protocol::SyntaxShape,
    optional: bool,
    rest: bool,
) -> String {
    let prefix = if rest { "..." } else { "" };
    let suffix = if optional { "?" } else { "" };

    match shape {
        nu_protocol::SyntaxShape::Any => format!("{prefix}{name}{suffix}"),
        _ => format!("{prefix}{name}{suffix}: {}", shape_to_string(shape)),
    }
}

fn shape_to_string(shape: &nu_protocol::SyntaxShape) -> String {
    use nu_protocol::SyntaxShape;

    match shape {
        SyntaxShape::Int => "int".into(),
        SyntaxShape::String => "string".into(),
        SyntaxShape::Float => "float".into(),
        SyntaxShape::Boolean => "bool".into(),
        SyntaxShape::List(inner) => format!("list<{}>", shape_to_string(inner)),
        SyntaxShape::Table(cols) if cols.is_empty() => "table".into(),
        SyntaxShape::Table(cols) => {
            let col_names = cols
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("table<{col_names}>")
        }
        SyntaxShape::Record(_) => "record".into(),
        SyntaxShape::Filepath => "path".into(),
        SyntaxShape::Directory => "directory".into(),
        SyntaxShape::GlobPattern => "glob".into(),
        SyntaxShape::Any => "any".into(),
        _ => format!("{shape:?}").to_lowercase(),
    }
}

fn check_def_call(call: &Call, ctx: &LintContext) -> Vec<RuleViolation> {
    let Some((block_id, func_name)) = call.extract_function_definition(ctx) else {
        return vec![];
    };

    let Some((_, name_span)) = call.extract_declaration_name(ctx) else {
        return vec![];
    };

    let block = ctx.working_set.get_block(block_id);
    let signature = &block.signature;
    let sig_span = find_signature_span(call, ctx);

    let uses_in = block_id.uses_pipeline_input(ctx);
    let produces_out = block_id.produces_output(ctx);
    let needs_input_type = uses_in && is_untyped(signature, sig_span, ctx, |(input, _)| input);
    let needs_output_type =
        produces_out && is_untyped(signature, sig_span, ctx, |(_, output)| output);

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
