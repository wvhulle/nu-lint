use nu_protocol::ast::{Argument, Block, Call, Expr, Expression, Pipeline};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::is_pipeline_input_var, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct ShebangFixData {
    fix_span: nu_protocol::Span,
    new_shebang: String,
}

fn uses_pipeline_input_directly(expr: &Expression, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::Var(var_id) => {
            let var = context.working_set.get_variable(*var_id);
            is_pipeline_input_var(*var_id, context) && var.const_val.is_none()
        }
        Expr::BinaryOp(left, _, right) => {
            uses_pipeline_input_directly(left, context)
                || uses_pipeline_input_directly(right, context)
        }
        Expr::UnaryNot(inner) => uses_pipeline_input_directly(inner, context),
        Expr::Collect(_var_id, _inner_expr) => true,
        Expr::Call(call) => call.arguments.iter().any(|arg| {
            if let Argument::Positional(arg_expr) | Argument::Named((_, _, Some(arg_expr))) = arg {
                uses_pipeline_input_directly(arg_expr, context)
            } else {
                false
            }
        }),
        Expr::FullCellPath(cell_path) => uses_pipeline_input_directly(&cell_path.head, context),
        Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block_uses_pipeline_input_directly(block, context)
        }
        _ => false,
    }
}

fn requires_stdin_from_signature(context: &LintContext, call: &Call) -> bool {
    let decl = context.working_set.get_decl(call.decl_id);
    let sig = decl.signature();

    if sig.input_output_types.is_empty() {
        return false;
    }

    let is_streaming_category = matches!(
        sig.category,
        nu_protocol::Category::Filters
            | nu_protocol::Category::Conversions
            | nu_protocol::Category::Formats
    );

    if !is_streaming_category {
        return false;
    }

    let requires_stdin = sig
        .input_output_types
        .iter()
        .all(|(input_type, _)| !matches!(input_type, nu_protocol::Type::Nothing));

    log::debug!(
        "Command '{}' (category: {:?}) requires stdin from signature: {}",
        decl.name(),
        sig.category,
        requires_stdin
    );

    requires_stdin
}

fn is_bare_streaming_command(pipeline: &Pipeline, context: &LintContext) -> bool {
    let Some(first_elem) = pipeline.elements.first() else {
        return false;
    };

    let Expr::Call(call) = &first_elem.expr.expr else {
        return false;
    };

    requires_stdin_from_signature(context, call)
}

fn block_uses_pipeline_input_directly(block: &Block, context: &LintContext) -> bool {
    block.pipelines.iter().any(|pipeline| {
        is_bare_streaming_command(pipeline, context)
            || pipeline
                .elements
                .iter()
                .any(|elem| uses_pipeline_input_directly(&elem.expr, context))
    })
}

fn has_stdin_flag_in_shebang(first_line: &str) -> bool {
    first_line.starts_with("#!") && first_line.contains("--stdin")
}

fn create_fix_data_for_shebang(context: &LintContext) -> Option<ShebangFixData> {
    let source = unsafe { context.source() };
    let file_offset = context.file_offset();
    let first_line = source.lines().next()?;

    if !first_line.starts_with("#!") {
        return None;
    }

    // Find the end of the first line including the newline
    let first_line_end = source.find('\n').map_or(source.len(), |pos| pos);
    let fix_span = nu_protocol::Span::new(file_offset, first_line_end + file_offset);

    let new_shebang = if first_line.contains("-S ") {
        first_line.replace("-S nu", "-S nu --stdin")
    } else if first_line.contains("env nu") {
        first_line.replace("env nu", "env -S nu --stdin")
    } else {
        format!("{first_line} --stdin")
    };

    Some(ShebangFixData {
        fix_span,
        new_shebang,
    })
}

fn has_explicit_type_annotation(
    signature_span: Option<nu_protocol::Span>,
    ctx: &LintContext,
) -> bool {
    signature_span.is_some_and(|span| span.source_code(ctx).contains("->"))
}

fn find_signature_span(call: &Call, _ctx: &LintContext) -> Option<nu_protocol::Span> {
    let sig_arg = call.get_positional_arg(1)?;
    Some(sig_arg.span)
}

fn check_main_function(
    call: &Call,
    context: &LintContext,
) -> Vec<(Detection, Option<ShebangFixData>)> {
    let (_func_name, name_span) = match call.extract_declaration_name(context) {
        Some((name, span)) if name == "main" => (name, span),
        _ => return vec![],
    };

    let Some(def) = call.custom_command_def(context) else {
        return vec![];
    };

    let block = context.working_set.get_block(def.body);
    let signature = &block.signature;
    let sig_span = find_signature_span(call, context);

    let uses_in = block_uses_pipeline_input_directly(block, context);

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

    let source = unsafe { context.source() };
    let Some(first_line) = source.lines().next() else {
        return vec![];
    };

    // Only check scripts with shebangs
    if !first_line.starts_with("#!") {
        return vec![];
    }

    if has_stdin_flag_in_shebang(first_line) {
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

    let fix_data = create_fix_data_for_shebang(context);

    let violation = Detection::from_global_span(message, name_span)
        .with_primary_label("main function expecting stdin")
        .with_help(
            "Add --stdin flag to shebang: #!/usr/bin/env -S nu --stdin or #!/usr/bin/env nu \
             --stdin (if env supports multiple args)",
        );

    vec![(violation, fix_data)]
}

struct MissingStdinInShebang;

impl DetectFix for MissingStdinInShebang {
    type FixInput<'a> = Option<ShebangFixData>;

    fn id(&self) -> &'static str {
        "missing_stdin_in_shebang"
    }

    fn explanation(&self) -> &'static str {
        "Scripts with main functions that expect pipeline input must have --stdin in shebang"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/scripts.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::Call(call) => check_main_function(call, ctx),
            _ => vec![],
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        fix_data.as_ref().map(|data| {
            Fix::with_explanation(
                "Add --stdin flag to shebang",
                vec![Replacement::new(data.fix_span, data.new_shebang.clone())],
            )
        })
    }
}

pub static RULE: &dyn Rule = &MissingStdinInShebang;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
