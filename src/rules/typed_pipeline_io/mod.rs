use nu_protocol::{
    Span,
    ast::{Argument, Block, Call, Expr, Expression},
};

use crate::{
    ast::call::CallExt,
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, RuleViolation, Severity},
};

fn block_uses_pipeline_input(block: &Block, context: &LintContext) -> bool {
    block.pipelines.iter().any(|pipeline| {
        pipeline
            .elements
            .iter()
            .any(|element| check_expr_for_in_variable(&element.expr, context))
    })
}

fn check_expr_for_in_variable(expr: &Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Expr;
    match &expr.expr {
        Expr::Var(var_id) => {
            let var = context.working_set.get_variable(*var_id);
            let span_start = var.declaration_span.start;
            let span_end = var.declaration_span.end;
            var.const_val.is_none() && span_start == 0 && span_end == 0
        }
        Expr::BinaryOp(left, _, right) => {
            check_expr_for_in_variable(left, context)
                || check_expr_for_in_variable(right, context)
        }
        Expr::UnaryNot(inner) => check_expr_for_in_variable(inner, context),
        Expr::Collect(_var_id, _inner_expr) => {
            true
        }
        Expr::Call(call) => call.arguments.iter().any(|arg| {
            if let Argument::Positional(arg_expr) | Argument::Named((_, _, Some(arg_expr))) = arg {
                check_expr_for_in_variable(arg_expr, context)
            } else {
                false
            }
        }),
        Expr::FullCellPath(cell_path) => check_expr_for_in_variable(&cell_path.head, context),
        Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block_uses_pipeline_input(block, context)
        }
        _ => false,
    }
}

fn has_untyped_pipeline_input(signature: &nu_protocol::Signature, signature_span: Option<Span>, ctx: &LintContext) -> bool {
    // First check if there's an explicit type annotation in the source
    if let Some(span) = signature_span {
        let sig_text = ctx.working_set.get_span_contents(span);
        let sig_str = String::from_utf8_lossy(sig_text);
        // If the signature contains "->", it has an explicit type annotation
        if sig_str.contains("->") {
            return false;
        }
    }
    
    // Otherwise, check if all types are Any (the default)
    signature.input_output_types.is_empty()
        || signature
            .input_output_types
            .iter()
            .all(|(input_type, _)| matches!(input_type, nu_protocol::Type::Any))
}

fn has_untyped_pipeline_output(signature: &nu_protocol::Signature, signature_span: Option<Span>, ctx: &LintContext) -> bool {
    // First check if there's an explicit type annotation in the source
    if let Some(span) = signature_span {
        let sig_text = ctx.working_set.get_span_contents(span);
        let sig_str = String::from_utf8_lossy(sig_text);
        // If the signature contains "->", it has an explicit type annotation
        if sig_str.contains("->") {
            return false;
        }
    }
    
    // Otherwise, check if all types are Any (the default)
    signature.input_output_types.is_empty()
        || signature
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
    fix: Fix,
) -> Vec<RuleViolation> {
    let mut violations = vec![];

    if needs_input_type {
        let suggestion = "Add pipeline input type annotation (e.g., `: string -> any` or `: \
                          list<int> -> any`)";
        violations.push(
            RuleViolation::new_dynamic(
                "typed_pipeline_io",
                format!(
                    "Custom command '{func_name}' uses pipeline input ($in) but lacks input type \
                     annotation"
                ),
                name_span,
            )
            .with_suggestion_static(suggestion)
            .with_fix(fix.clone()),
        );
    }

    if needs_output_type {
        let suggestion = if uses_in {
            "Add pipeline output type annotation (e.g., `: any -> string` or `: list<int> -> \
             table`)"
        } else {
            "Add pipeline output type annotation (e.g., `: nothing -> string` or `: nothing -> \
             list<int>`)"
        };
        violations.push(
            RuleViolation::new_dynamic(
                "typed_pipeline_io",
                format!(
                    "Custom command '{func_name}' produces output but lacks output type \
                     annotation"
                ),
                name_span,
            )
            .with_suggestion_static(suggestion)
            .with_fix(fix),
        );
    }

    violations
}

fn generate_typed_signature(
    signature: &nu_protocol::Signature,
    _ctx: &LintContext,
    uses_in: bool,
    needs_input_type: bool,
    needs_output_type: bool,
) -> String {
    let current_sig = signature;

    let params_text = if current_sig.required_positional.is_empty()
        && current_sig.optional_positional.is_empty()
        && current_sig.rest_positional.is_none()
        && current_sig.named.is_empty()
    {
        String::new()
    } else {
        extract_parameters_text(signature)
    };

    let input_type = if uses_in || needs_input_type {
        "any"
    } else {
        "nothing"
    };

    let output_type = "any";

    if needs_input_type || needs_output_type {
        format!("[{params_text}]: {input_type} -> {output_type}")
    } else {
        format!("[{params_text}]")
    }
}

fn extract_parameters_text(signature: &nu_protocol::Signature) -> String {
    let mut params = Vec::new();

    for param in &signature.required_positional {
        let type_str = if param.shape == nu_protocol::SyntaxShape::Any {
            String::new()
        } else {
            format!(": {}", shape_to_string(&param.shape))
        };
        params.push(format!("{}{type_str}", param.name));
    }

    for param in &signature.optional_positional {
        let type_str = if param.shape == nu_protocol::SyntaxShape::Any {
            String::new()
        } else {
            format!(": {}", shape_to_string(&param.shape))
        };
        params.push(format!("{}?{type_str}", param.name));
    }

    if let Some(rest) = &signature.rest_positional {
        let type_str = if rest.shape == nu_protocol::SyntaxShape::Any {
            String::new()
        } else {
            format!(": {}", shape_to_string(&rest.shape))
        };
        params.push(format!("...{}{type_str}", rest.name));
    }

    for flag in &signature.named {
        if flag.long == "help" {
            continue;
        }
        
        if let Some(short) = flag.short {
            if let Some(arg_shape) = &flag.arg {
                params.push(format!(
                    "--{} (-{}): {}",
                    flag.long,
                    short,
                    shape_to_string(arg_shape)
                ));
            } else {
                params.push(format!("--{} (-{})", flag.long, short));
            }
        } else if let Some(arg_shape) = &flag.arg {
            params.push(format!("--{}: {}", flag.long, shape_to_string(arg_shape)));
        } else {
            params.push(format!("--{}", flag.long));
        }
    }

    params.join(", ")
}

fn shape_to_string(shape: &nu_protocol::SyntaxShape) -> String {
    match shape {
        nu_protocol::SyntaxShape::Int => "int".to_string(),
        nu_protocol::SyntaxShape::String => "string".to_string(),
        nu_protocol::SyntaxShape::Float => "float".to_string(),
        nu_protocol::SyntaxShape::Boolean => "bool".to_string(),
        nu_protocol::SyntaxShape::List(inner) => format!("list<{}>", shape_to_string(inner)),
        nu_protocol::SyntaxShape::Table(cols) => {
            if cols.is_empty() {
                "table".to_string()
            } else {
                let col_names: Vec<_> = cols.iter().map(|(name, _)| name.as_str()).collect();
                format!("table<{}>", col_names.join(", "))
            }
        }
        nu_protocol::SyntaxShape::Record(_fields) => "record".to_string(),
        nu_protocol::SyntaxShape::Filepath => "path".to_string(),
        nu_protocol::SyntaxShape::Directory => "directory".to_string(),
        nu_protocol::SyntaxShape::GlobPattern => "glob".to_string(),
        nu_protocol::SyntaxShape::Any => "any".to_string(),
        _ => format!("{shape:?}").to_lowercase(),
    }
}

fn check_def_call(call: &Call, ctx: &LintContext) -> Vec<RuleViolation> {
    let decl = ctx.working_set.get_decl(call.decl_id);
    if decl.name() != "def" && decl.name() != "export def" {
        return vec![];
    }

    let Some((func_name, name_span)) = call.extract_declaration_name(ctx) else {
        return vec![];
    };

    let Some((block_id, _)) = call.extract_function_definition(ctx) else {
        return vec![];
    };

    let block = ctx.working_set.get_block(block_id);
    let signature = &block.signature;

    let mut violations = vec![];

    let sig_span = find_signature_span(call, ctx);
    let uses_in = block_uses_pipeline_input(block, ctx);
    let has_untyped_input = has_untyped_pipeline_input(signature, sig_span, ctx);
    let has_untyped_output = has_untyped_pipeline_output(signature, sig_span, ctx);
    let produces_out = produces_output(block, ctx);

    if (uses_in && has_untyped_input) || (produces_out && has_untyped_output) {
        let needs_input_type = uses_in && has_untyped_input;
        let needs_output_type = produces_out && has_untyped_output;

        let new_signature = generate_typed_signature(
            signature,
            ctx,
            uses_in,
            needs_input_type,
            needs_output_type,
        );

        if let Some(sig_span) = sig_span {
            let fix = Fix::new_dynamic(
                format!("Add type annotations: {new_signature}"),
                vec![Replacement::new_dynamic(sig_span, new_signature)],
            );

            violations.extend(create_violations_for_untyped_io(
                &func_name,
                name_span,
                uses_in,
                needs_input_type,
                needs_output_type,
                fix,
            ));
        }
    }

    violations
}

fn produces_output(block: &Block, _context: &LintContext) -> bool {
    if block.pipelines.is_empty() {
        return false;
    }

    block.pipelines.iter().any(|pipeline| {
        pipeline
            .elements
            .last()
            .is_some_and(|last_element| !matches!(&last_element.expr.expr, Expr::Nothing))
    })
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
        Severity::Info,
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
