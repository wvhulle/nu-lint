use nu_protocol::{
    Span, Type,
    ast::{
        Block, Expr, Pipeline, PipelineElement, PipelineRedirection, RedirectionTarget, Traverse,
    },
};

use crate::{
    Fix, Replacement,
    ast::expression::ExpressionExt,
    config::LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct StructuredDataToExternal;

/// Represents the conversion format needed for piping data to external commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExternalStdin {
    /// Convert to JSON format
    Json,
    /// Convert to CSV format
    Csv,
    /// Convert to plain text
    Text,

    /// Join list elements with newlines
    Lines,
}

impl ExternalStdin {
    /// Determine the best conversion format based on external command and data
    /// type
    fn from_command_and_type(cmd_name: &str, ty: &Type) -> Self {
        // First check if the command has a known preferred format
        if let Some(format) = Self::get_command_preference(cmd_name, ty) {
            return format;
        }

        // Otherwise, use type-based defaults
        Self::from_type(ty)
    }

    /// Get the preferred format for known external commands
    fn get_command_preference(cmd_name: &str, ty: &Type) -> Option<Self> {
        // Extract command name without path or arguments
        let cmd_base = cmd_name
            .trim_start_matches("./")
            .trim_start_matches("../")
            .split('/')
            .next_back()
            .unwrap_or(cmd_name)
            .split_whitespace()
            .next()
            .unwrap_or(cmd_name);

        match cmd_base {
            // Commands that expect JSON (regardless of data type)
            "jq" | "json_pp" | "json" | "jsonlint" => Some(Self::Json),

            // Commands that expect CSV (for tables)
            "csvlook" | "csvstat" | "csvcut" | "csvgrep" => Some(Self::Csv),

            // Line-oriented commands: use Lines for lists, Text for tables
            "grep" | "sed" | "awk" | "sort" | "uniq" | "wc" | "cat" | "less" | "more" | "head"
            | "tail" => match ty {
                Type::List(_) => Some(Self::Lines),
                Type::Table(_) | Type::Record(_) => Some(Self::Text),
                _ => None, // Let from_type handle primitives
            },

            // No specific preference for other commands
            _ => None,
        }
    }
    /// Get default format based on data type
    const fn from_type(ty: &Type) -> Self {
        match ty {
            // Tables and records: JSON is more useful for structured data going to unknown commands
            Type::Table(_) | Type::Record(_) => Self::Json,
            // Lists can be joined into lines
            Type::List(_) => Self::Lines,
            // Primitives need to be converted to text
            _ => Self::Text,
        }
    }

    /// Get the Nu command string for this conversion
    const fn as_command(self) -> &'static str {
        match self {
            Self::Json => "to json",
            Self::Csv => "to csv",
            Self::Text => "to text",
            Self::Lines => "str join (char newline)",
        }
    }

    /// Get a human-readable description of this format
    const fn description(self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Csv => "CSV",
            Self::Text => "plain text",
            Self::Lines => "newline-separated text",
        }
    }
}

#[derive(Debug)]
struct FixData {
    /// Span of the expression producing structured data
    data_span: Span,

    /// The conversion format to use
    conversion: ExternalStdin,
}

impl DetectFix for StructuredDataToExternal {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "convert_structured_to_string_external"
    }

    fn explanation(&self) -> &'static str {
        "Don't pipe structured data (tables, records, lists) directly into external commands."
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/pipelines.html#external-commands")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();
        check_block_recursive(context.ast, context, &mut violations);
        violations
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let original_text = context.plain_text(fix_data.data_span);

        let new_text = format!("{} | {}", original_text, fix_data.conversion.as_command());

        let explanation = format!(
            "Add '{}' to convert to {} before piping to external command",
            fix_data.conversion.as_command(),
            fix_data.conversion.description()
        );

        Some(Fix::with_explanation(
            explanation,
            vec![Replacement::new(fix_data.data_span, new_text)],
        ))
    }
}

/// Recursively check a block and all its nested blocks for violations
fn check_block_recursive(
    block: &Block,
    context: &LintContext,
    violations: &mut Vec<(Detection, FixData)>,
) {
    for pipeline in &block.pipelines {
        violations.extend(check_pipeline(pipeline, context));
    }

    // Find and recursively check all nested blocks
    let mut nested_block_ids = Vec::new();
    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            element.expr.flat_map(
                context.working_set,
                &|expr| match &expr.expr {
                    Expr::Block(id)
                    | Expr::RowCondition(id)
                    | Expr::Closure(id)
                    | Expr::Subexpression(id) => vec![*id],
                    _ => vec![],
                },
                &mut nested_block_ids,
            );
        }
    }

    for &block_id in &nested_block_ids {
        check_block_recursive(context.working_set.get_block(block_id), context, violations);
    }
}

/// Check a single pipeline for structured data piped to external commands
fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    pipeline
        .elements
        .windows(2)
        .filter_map(|window| check_pipeline_pair(&window[0], &window[1], context))
        .collect()
}

/// Check if left element pipes structured data into right element's external
/// command
fn check_pipeline_pair(
    left: &PipelineElement,
    right: &PipelineElement,
    context: &LintContext,
) -> Option<(Detection, FixData)> {
    // Check if right is an external call
    let Expr::ExternalCall(head, _args) = &right.expr.expr else {
        return None;
    };

    let cmd_name = context.plain_text(head.span);

    log::debug!(
        "Checking external call to '{}' with input from '{}'",
        cmd_name,
        context.plain_text(left.expr.span)
    );

    // Infer what's being piped in
    let input_type = left.expr.infer_output_type(context)?;

    log::debug!("Inferred input type: {input_type:?}");

    // Check if it's structured data
    is_structured_type(&input_type).then(|| {
        let conversion = ExternalStdin::from_command_and_type(cmd_name, &input_type);
        create_violation(&input_type, cmd_name, left, right, conversion)
    })
}

/// Check if a type is structured data that shouldn't be piped to external
/// commands
const fn is_structured_type(ty: &Type) -> bool {
    match ty {
        // Structured types that will be auto-serialized
        Type::Table(_)
        | Type::Record(_)
        | Type::List(_)
        | Type::Int
        | Type::Float
        | Type::Bool
        | Type::Date
        | Type::Duration
        | Type::Filesize
        | Type::Range => true,

        // Other types - be conservative and allow
        _ => false,
    }
}

fn create_violation(
    ty: &Type,
    cmd_name: &str,
    left_element: &PipelineElement,
    right_element: &PipelineElement,
    conversion: ExternalStdin,
) -> (Detection, FixData) {
    let type_name = ty.to_string();

    let message = format!(
        "Piping {type_name} into external command '{cmd_name}' will auto-serialize to string"
    );

    let mut detection = Detection::from_global_span(message, right_element.expr.span)
        .with_extra_label(format!("{type_name} output"), left_element.expr.span)
        .with_extra_label("external command", right_element.expr.span);

    // Add redirection label if present
    if let Some(ref redir) = left_element.redirection {
        detection = detection.with_extra_label("with redirection", get_redirection_span(redir));
    }

    detection = detection.with_help(
        "External commands receive data as text. Structured data will be implicitly serialized as \
         string input for the external command."
            .to_string(),
    );

    let fix_data = FixData {
        data_span: left_element.expr.span,
        conversion,
    };

    (detection, fix_data)
}

const fn get_redirection_span(redir: &PipelineRedirection) -> Span {
    match redir {
        PipelineRedirection::Single { target, .. } => match target {
            RedirectionTarget::File { expr, .. } => expr.span,
            RedirectionTarget::Pipe { .. } => Span::unknown(),
        },
        PipelineRedirection::Separate { out, .. } => match out {
            RedirectionTarget::File { expr, .. } => expr.span,
            RedirectionTarget::Pipe { .. } => Span::unknown(),
        },
    }
}

pub static RULE: &dyn Rule = &StructuredDataToExternal;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
