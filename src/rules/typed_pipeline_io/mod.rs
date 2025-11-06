use nu_protocol::{
    Span,
    ast::{Call, Expr},
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
        && (signature.input_output_types.is_empty()
            || signature
                .input_output_types
                .iter()
                .all(|(input_type, _)| matches!(input_type, nu_protocol::Type::Any)))
}

fn has_untyped_pipeline_output(
    signature: &nu_protocol::Signature,
    signature_span: Option<Span>,
    ctx: &LintContext,
) -> bool {
    !has_explicit_type_annotation(signature_span, ctx)
        && (signature.input_output_types.is_empty()
            || signature
                .input_output_types
                .iter()
                .all(|(_, output_type)| matches!(output_type, nu_protocol::Type::Any)))
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
    let mut violations = Vec::new();

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
            .with_fix(fix.clone()),
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
    let params_text = if signature.required_positional.is_empty()
        && signature.optional_positional.is_empty()
        && signature.rest_positional.is_none()
        && signature.named.is_empty()
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

    if needs_input_type || needs_output_type {
        format!("[{params_text}]: {input_type} -> any")
    } else {
        format!("[{params_text}]")
    }
}

fn extract_parameters_text(signature: &nu_protocol::Signature) -> String {
    let mut params = Vec::new();

    params.extend(signature.required_positional.iter().map(|param| {
        if param.shape == nu_protocol::SyntaxShape::Any {
            param.name.clone()
        } else {
            format!("{}: {}", param.name, shape_to_string(&param.shape))
        }
    }));

    params.extend(signature.optional_positional.iter().map(|param| {
        if param.shape == nu_protocol::SyntaxShape::Any {
            format!("{}?", param.name)
        } else {
            format!("{}?: {}", param.name, shape_to_string(&param.shape))
        }
    }));

    if let Some(rest) = &signature.rest_positional {
        let param = if rest.shape == nu_protocol::SyntaxShape::Any {
            format!("...{}", rest.name)
        } else {
            format!("...{}: {}", rest.name, shape_to_string(&rest.shape))
        };
        params.push(param);
    }

    params.extend(
        signature
            .named
            .iter()
            .filter(|flag| flag.long != "help")
            .map(|flag| match (&flag.short, &flag.arg) {
                (Some(short), Some(arg_shape)) => {
                    format!("--{} (-{}): {}", flag.long, short, shape_to_string(arg_shape))
                }
                (Some(short), None) => format!("--{} (-{})", flag.long, short),
                (None, Some(arg_shape)) => {
                    format!("--{}: {}", flag.long, shape_to_string(arg_shape))
                }
                (None, None) => format!("--{}", flag.long),
            }),
    );

    params.join(", ")
}

fn shape_to_string(shape: &nu_protocol::SyntaxShape) -> String {
    use nu_protocol::SyntaxShape;
    
    match shape {
        SyntaxShape::Int => "int",
        SyntaxShape::String => "string",
        SyntaxShape::Float => "float",
        SyntaxShape::Boolean => "bool",
        SyntaxShape::List(inner) => return format!("list<{}>", shape_to_string(inner)),
        SyntaxShape::Table(cols) if cols.is_empty() => "table",
        SyntaxShape::Table(cols) => {
            let col_names: Vec<_> = cols.iter().map(|(name, _)| name.as_str()).collect();
            return format!("table<{}>", col_names.join(", "));
        }
        SyntaxShape::Record(_) => "record",
        SyntaxShape::Filepath => "path",
        SyntaxShape::Directory => "directory",
        SyntaxShape::GlobPattern => "glob",
        SyntaxShape::Any => "any",
        _ => return format!("{shape:?}").to_lowercase(),
    }
    .to_string()
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
    let has_untyped_input = has_untyped_pipeline_input(signature, sig_span, ctx);
    let has_untyped_output = has_untyped_pipeline_output(signature, sig_span, ctx);
    let produces_out = block_id.produces_output(ctx);

    let needs_input_type = uses_in && has_untyped_input;
    let needs_output_type = produces_out && has_untyped_output;

    if !needs_input_type && !needs_output_type {
        return vec![];
    }

    let Some(sig_span) = sig_span else {
        return vec![];
    };

    let new_signature =
        generate_typed_signature(signature, ctx, uses_in, needs_input_type, needs_output_type);

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
