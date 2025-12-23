//! Rule: `prefer_multiline_string`
//!
//! Detects consecutive `print` statements with string literals or string
//! interpolations and suggests merging them into a single `print` with a
//! multiline string.

use std::mem;

use nu_protocol::{
    Span,
    ast::{Argument, Block, Call, Expr, Pipeline, Traverse},
};

use crate::{
    ast::call::CallExt,
    config::LintLevel,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

const MIN_CONSECUTIVE_PRINTS: usize = 3;

/// The type of string content in a print statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringType {
    /// Plain string literal: `"text"` or `'text'`
    Plain,
    /// String interpolation: `$"text ($var)"` or `$'text ($var)'`
    Interpolation,
}

/// Information extracted from a single `print` statement.
#[derive(Debug, Clone)]
struct PrintInfo {
    span: Span,
    /// The inner content of the string (without quotes/interpolation markers)
    content: String,
    to_stderr: bool,
    string_type: StringType,
}

impl PrintInfo {
    /// Attempts to extract print info from a pipeline.
    ///
    /// Returns `None` if the pipeline is not a simple `print "string"` or
    /// `print $"interpolation"` call.
    fn from_pipeline(pipeline: &Pipeline, context: &LintContext) -> Option<Self> {
        let [element] = pipeline.elements.as_slice() else {
            return None;
        };

        let Expr::Call(call) = &element.expr.expr else {
            return None;
        };

        if !call.is_call_to_command("print", context) {
            return None;
        }

        let to_stderr = call.has_named_flag("stderr") || call.has_named_flag("e");
        let (string_content, string_type) = Self::extract_string_content(call, context)?;

        Some(Self {
            span: element.expr.span,
            content: string_content,
            to_stderr,
            string_type,
        })
    }

    /// Extracts the string content and type from a print call's first
    /// positional argument.
    fn extract_string_content(call: &Call, context: &LintContext) -> Option<(String, StringType)> {
        let expr = call.arguments.iter().find_map(|arg| match arg {
            Argument::Positional(e) | Argument::Unknown(e) => Some(e),
            _ => None,
        })?;

        match &expr.expr {
            Expr::String(s) | Expr::RawString(s) => Some((s.clone(), StringType::Plain)),
            Expr::StringInterpolation(_) => {
                let text = context.get_span_text(expr.span);
                Self::parse_interpolation_string(text)
            }
            _ => {
                let text = context.get_span_text(expr.span);
                Self::parse_quoted_string(text).map(|s| (s, StringType::Plain))
            }
        }
    }

    /// Parses a quoted string, returning its content without quotes.
    fn parse_quoted_string(text: &str) -> Option<String> {
        let is_double_quoted = text.starts_with('"') && text.ends_with('"');
        let is_single_quoted = text.starts_with('\'') && text.ends_with('\'');

        (is_double_quoted || is_single_quoted).then(|| text[1..text.len() - 1].to_string())
    }

    /// Parses a string interpolation, returning its inner content.
    /// Input: `$"Hello ($name)"` -> Output: `Hello ($name)`
    fn parse_interpolation_string(text: &str) -> Option<(String, StringType)> {
        let content = text
            .strip_prefix("$\"")
            .and_then(|s| s.strip_suffix('"'))
            .or_else(|| text.strip_prefix("$'").and_then(|s| s.strip_suffix('\'')))?;

        Some((content.to_string(), StringType::Interpolation))
    }
}

/// Groups consecutive print statements that share the same output stream.
struct PrintGrouper<'a> {
    context: &'a LintContext<'a>,
}

impl<'a> PrintGrouper<'a> {
    const fn new(context: &'a LintContext<'a>) -> Self {
        Self { context }
    }

    /// Finds all groups of consecutive print statements in a block.
    fn find_groups(&self, block: &Block) -> Vec<Vec<PrintInfo>> {
        let mut groups = Vec::new();
        let mut current_group: Vec<PrintInfo> = Vec::new();

        for pipeline in &block.pipelines {
            match PrintInfo::from_pipeline(pipeline, self.context) {
                Some(info) if Self::can_extend_group(&current_group, &info) => {
                    current_group.push(info);
                }
                Some(info) => {
                    Self::flush_group(&mut groups, &mut current_group);
                    current_group.push(info);
                }
                None => {
                    Self::flush_group(&mut groups, &mut current_group);
                }
            }
        }

        Self::flush_group(&mut groups, &mut current_group);
        groups
    }

    /// Checks if a print info can extend the current group.
    /// Groups must have same stderr flag AND same string type.
    fn can_extend_group(group: &[PrintInfo], info: &PrintInfo) -> bool {
        group.first().is_none_or(|first| {
            first.to_stderr == info.to_stderr && first.string_type == info.string_type
        })
    }

    /// Flushes the current group to the groups list if it meets the threshold.
    fn flush_group(groups: &mut Vec<Vec<PrintInfo>>, current: &mut Vec<PrintInfo>) {
        if current.len() >= MIN_CONSECUTIVE_PRINTS {
            groups.push(mem::take(current));
        } else {
            current.clear();
        }
    }
}

/// Creates a violation for a group of consecutive print statements.
fn create_violation(prints: &[PrintInfo]) -> Violation {
    let combined_span = Span::new(
        prints.first().map_or(0, |p| p.span.start),
        prints.last().map_or(0, |p| p.span.end),
    );

    // Join content with actual newlines for a multiline string
    let merged_content = prints
        .iter()
        .map(|p| p.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let stderr_flag = prints.first().filter(|p| p.to_stderr).map_or("", |_| " -e");

    // Use $"..." for interpolation, "..." for plain strings
    let is_interpolation = prints
        .first()
        .is_some_and(|p| p.string_type == StringType::Interpolation);

    let replacement_text = if is_interpolation {
        format!("print{stderr_flag} $\"{merged_content}\"")
    } else {
        format!("print{stderr_flag} \"{merged_content}\"")
    };

    let fix = Fix::with_explanation(
        format!(
            "Merge {} consecutive print statements into a single multiline print",
            prints.len()
        ),
        vec![Replacement::new(combined_span, replacement_text)],
    );

    Violation::new(
        format!(
            "Found {} consecutive `print` statements with string literals that could be merged",
            prints.len()
        ),
        combined_span,
    )
    .with_primary_label("consecutive prints")
    .with_help(
        "Merge consecutive print statements into a single print with a multiline string for \
         cleaner code. Use `print \"line1\\nline2\\nline3\"` instead of multiple print calls.",
    )
    .with_fix(fix)
}

/// Recursively checks a block and all nested blocks for violations.
fn check_block(block: &Block, context: &LintContext) -> Vec<Violation> {
    let grouper = PrintGrouper::new(context);
    let mut violations: Vec<_> = grouper
        .find_groups(block)
        .iter()
        .map(|group| create_violation(group))
        .collect();

    // Check nested blocks (closures, subexpressions, etc.)
    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            element.expr.flat_map(
                context.working_set,
                &|expr| match &expr.expr {
                    Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => {
                        check_block(context.working_set.get_block(*id), context)
                    }
                    _ => vec![],
                },
                &mut violations,
            );
        }
    }

    violations
}

fn check(context: &LintContext) -> Vec<Violation> {
    check_block(context.ast, context)
}

pub const fn rule() -> Rule {
    Rule::new(
        "merge_multiline_print",
        "Consecutive print statements with string literals should be merged into a single print \
         with a multiline string",
        check,
        LintLevel::Hint,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
