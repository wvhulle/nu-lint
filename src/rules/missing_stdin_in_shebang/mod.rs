use nu_protocol::ast::{Call, Expr};

use crate::{
    ast::{block::BlockExt, call::CallExt, span::SpanExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

fn has_stdin_flag_in_shebang(source: &str) -> bool {
    source
        .lines()
        .next()
        .is_some_and(|first_line| first_line.starts_with("#!") && first_line.contains("--stdin"))
}

fn create_fix_for_shebang(source: &str) -> Option<crate::Fix> {
    let first_line = source.lines().next()?;
    if !first_line.starts_with("#!") {
        return None;
    }

    // Find the end of the first line including the newline
    let first_line_end = source.find('\n').map_or(source.len(), |pos| pos);
    let span = nu_protocol::Span::new(0, first_line_end);

    let new_shebang = if first_line.contains("-S ") {
        first_line.replace("-S nu", "-S nu --stdin")
    } else if first_line.contains("env nu") {
        first_line.replace("env nu", "env -S nu --stdin")
    } else {
        format!("{first_line} --stdin")
    };

    Some(crate::Fix::with_explanation(
        "Add --stdin flag to shebang",
        vec![crate::Replacement::new(span, new_shebang)],
    ))
}

fn has_explicit_type_annotation(
    signature_span: Option<nu_protocol::Span>,
    ctx: &LintContext,
) -> bool {
    signature_span.is_some_and(|span| span.text(ctx).contains("->"))
}

fn find_signature_span(call: &Call, _ctx: &LintContext) -> Option<nu_protocol::Span> {
    let sig_arg = call.get_positional_arg(1)?;
    Some(sig_arg.span)
}

fn check_main_function(call: &Call, context: &LintContext) -> Vec<Violation> {
    let (_func_name, name_span) = match call.extract_declaration_name(context) {
        Some((name, span)) if name == "main" => (name, span),
        _ => return vec![],
    };

    let Some((block_id, _)) = call.extract_function_definition(context) else {
        return vec![];
    };

    let block = context.working_set.get_block(block_id);
    let signature = &block.signature;
    let sig_span = find_signature_span(call, context);

    let uses_in = block.uses_pipeline_input(context);

    let has_explicit_annotation = has_explicit_type_annotation(sig_span, context);

    let has_non_nothing_input = has_explicit_annotation
        && signature
            .input_output_types
            .iter()
            .any(|(input, _)| !matches!(input, nu_protocol::Type::Nothing));

    let needs_stdin = uses_in || has_non_nothing_input;

    if !needs_stdin {
        return vec![];
    }

    if has_stdin_flag_in_shebang(context.source) {
        return vec![];
    }

    let message = if uses_in && has_non_nothing_input {
        "Main function uses $in and declares pipeline input type but shebang is missing --stdin \
         flag"
    } else if uses_in {
        "Main function uses $in variable but shebang is missing --stdin flag"
    } else {
        "Main function declares pipeline input type but shebang is missing --stdin flag"
    };

    let fix = create_fix_for_shebang(context.source);

    let mut violation = Violation::new("missing_stdin_in_shebang", message, name_span).with_help(
        "Add --stdin flag to shebang: #!/usr/bin/env -S nu --stdin or #!/usr/bin/env nu --stdin \
         (if env supports multiple args)",
    );

    if let Some(fix) = fix {
        violation = violation.with_fix(fix);
    }

    vec![violation]
}

fn check(context: &LintContext) -> Vec<Violation> {
    let has_shebang = context
        .source
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("#!"));

    if !has_shebang {
        return vec![];
    }

    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) => check_main_function(call, ctx),
        _ => vec![],
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "missing_stdin_in_shebang",
        "Scripts with main functions that expect pipeline input must have --stdin in shebang",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
