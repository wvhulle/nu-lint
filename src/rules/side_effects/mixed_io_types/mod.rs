use std::{collections::HashMap, fmt, string::ToString};

use nu_protocol::{
    BlockId, Span,
    ast::{Block, Call, Expr, Expression, ExternalArgument},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    effect::{
        builtin::{BuiltinEffect, has_builtin_side_effect},
        external::{ExternEffect, has_external_side_effect},
    },
    rule::{DetectFix, Rule},
    violation::Detection,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum IoType {
    FileSystem,
    Network,
    PrintStdout,
}

impl fmt::Display for IoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileSystem => write!(f, "file I/O"),

            Self::Network => write!(f, "network I/O"),
            Self::PrintStdout => write!(f, "print to stdout"),
        }
    }
}

fn classify_builtin_io(
    call: &Call,
    context: &LintContext,
    io_spans: &mut HashMap<IoType, Vec<Span>>,
) {
    let command_name = call.get_call_name(context);
    let category = context
        .working_set
        .get_decl(call.decl_id)
        .signature()
        .category;

    if has_builtin_side_effect(&command_name, BuiltinEffect::PrintToStdout, context, call) {
        io_spans
            .entry(IoType::PrintStdout)
            .or_default()
            .push(call.head);
    }

    match category {
        nu_protocol::Category::FileSystem => {
            io_spans
                .entry(IoType::FileSystem)
                .or_default()
                .push(call.head);
        }
        nu_protocol::Category::Network => {
            io_spans.entry(IoType::Network).or_default().push(call.head);
        }
        _ => {}
    }
}

fn matches_external_io_type(
    io_type: IoType,
    command_name: &str,
    args: &[ExternalArgument],
    context: &LintContext,
) -> bool {
    match io_type {
        IoType::Network => has_external_side_effect(
            command_name,
            ExternEffect::ModifiesNetworkState,
            context,
            args,
        ),
        IoType::FileSystem => has_external_side_effect(
            command_name,
            ExternEffect::ModifiesFileSystem,
            context,
            args,
        ),
        IoType::PrintStdout => {
            !has_external_side_effect(command_name, ExternEffect::NoDataInStdout, context, args)
        }
    }
}

fn classify_external_io(
    command_name: &str,
    args: &[ExternalArgument],
    head_span: Span,
    context: &LintContext,
    io_spans: &mut HashMap<IoType, Vec<Span>>,
) {
    for io_type in [IoType::Network, IoType::FileSystem] {
        if matches_external_io_type(io_type, command_name, args, context) {
            log::debug!(
                "External command '{command_name}' matches IoType::{io_type:?}, adding span \
                 {head_span:?}"
            );
            io_spans.entry(io_type).or_default().push(head_span);
        }
    }
}

fn collect_io_types_from_expression(
    expr: &Expression,
    context: &LintContext,
    io_spans: &mut HashMap<IoType, Vec<Span>>,
) {
    match &expr.expr {
        Expr::Call(call) => {
            classify_builtin_io(call, context, io_spans);
            for arg_expr in call.all_arg_expressions() {
                if let Some(block_id) = arg_expr.extract_block_id() {
                    let block = context.working_set.get_block(block_id);
                    let nested_io_spans = collect_io_types_from_block(block, context);
                    for (io_type, spans) in nested_io_spans {
                        io_spans.entry(io_type).or_default().extend(spans);
                    }
                } else {
                    collect_io_types_from_expression(arg_expr, context, io_spans);
                }
            }
        }
        Expr::ExternalCall(head, args) => {
            let command_name = context.get_span_text(head.span);
            classify_external_io(command_name, args, head.span, context, io_spans);
        }
        _ => {}
    }
}

fn collect_io_types_from_block(block: &Block, context: &LintContext) -> HashMap<IoType, Vec<Span>> {
    let mut io_spans = HashMap::new();

    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            collect_io_types_from_expression(&element.expr, context, &mut io_spans);
        }
    }

    io_spans
}

fn analyze_top_level_script(context: &LintContext) -> Option<Detection> {
    let io_spans = collect_io_types_from_block(context.ast, context);

    if io_spans.len() < 2 {
        return None;
    }

    let io_type_names: Vec<String> = io_spans.keys().map(ToString::to_string).collect();

    let message = format!(
        "Script mixes different I/O types: {}",
        io_type_names.join(", ")
    );

    let script_span = context.ast.span.unwrap_or(Span::unknown());

    let mut detection = Detection::from_global_span(message, script_span)
        .with_primary_label("script with mixed I/O")
        .with_help(
            "Consider separating different I/O operations into focused functions. This makes the \
             code easier to test, mock, and reason about. For scripts without functions, create \
             separate functions for network operations, file operations, and printing.",
        );

    for (io_type, spans) in &io_spans {
        for span in spans {
            detection = detection.with_extra_label(io_type.to_string(), *span);
        }
    }

    Some(detection)
}

fn analyze_function_body(
    block_id: BlockId,
    function_name: &str,
    context: &LintContext,
) -> Option<Detection> {
    let block = context.working_set.get_block(block_id);
    let io_spans = collect_io_types_from_block(block, context);

    if io_spans.len() < 2 {
        return None;
    }

    let io_type_names: Vec<String> = io_spans.keys().map(ToString::to_string).collect();

    let message = format!(
        "Function `{function_name}` mixes different I/O types: {}",
        io_type_names.join(", ")
    );

    let mut detection =
        Detection::from_file_span(message, context.find_declaration_span(function_name))
            .with_primary_label("function with mixed I/O")
            .with_help(
                "Consider separating different I/O operations into focused functions. This makes \
                 the code easier to test, mock, and reason about. Group file operations together, \
                 network operations together, and printing separately.",
            );

    for (io_type, spans) in &io_spans {
        for span in spans {
            detection = detection.with_extra_label(io_type.to_string(), *span);
        }
    }

    Some(detection)
}

struct SeparateLocalRemoteIo;

impl DetectFix for SeparateLocalRemoteIo {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "dont_mix_different_effects"
    }

    fn explanation(&self) -> &'static str {
        "Functions should not mix different types of I/O operations or effects."
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();

        let function_definitions = context.collect_function_definitions();

        if function_definitions.is_empty()
            && let Some(detection) = analyze_top_level_script(context)
        {
            violations.push(detection);
        }

        violations.extend(
            function_definitions
                .iter()
                .filter(|(_, name)| *name != "main")
                .filter_map(|(block_id, name)| analyze_function_body(*block_id, name, context)),
        );

        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &SeparateLocalRemoteIo;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
