//! Pipeline utilities and pattern detection helpers for lint rules.
//!
//! This module provides utilities for working with Nu pipelines,
//! including type inference and finding common patterns like
//! consecutive commands, command pairs, and command clusters.

use nu_protocol::{
    Span, Type, VarId,
    ast::{Call, Expr, Pipeline, PipelineElement},
};

use super::call::CallExt;
use crate::{ast::expression::ExpressionExt, context::LintContext};

/// Result of finding consecutive commands in a pipeline.
#[derive(Debug, Clone)]
pub struct CommandCluster<'a> {
    /// Indices of the commands in the pipeline
    pub indices: Vec<usize>,
    /// References to the actual call expressions
    pub calls: Vec<&'a Call>,
    /// Combined span from first to last element
    pub span: Span,
}

impl CommandCluster<'_> {
    /// Number of commands in the cluster
    #[must_use]
    pub const fn len(&self) -> usize {
        self.calls.len()
    }

    /// First index in the pipeline
    #[must_use]
    pub fn first_index(&self) -> Option<usize> {
        self.indices.first().copied()
    }

    /// Last index in the pipeline
    #[must_use]
    pub fn last_index(&self) -> Option<usize> {
        self.indices.last().copied()
    }
}

/// Configuration for finding command clusters.
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    /// Minimum number of consecutive commands to form a cluster
    pub min_size: usize,
    /// Maximum gap between commands (commands in between that don't match)
    pub max_gap: usize,
    /// Commands that can appear in gaps without breaking the cluster
    pub allowed_gap_commands: Vec<&'static str>,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            min_size: 2,
            max_gap: 0,
            allowed_gap_commands: Vec::new(),
        }
    }
}

impl ClusterConfig {
    /// Create config requiring at least `n` consecutive commands
    #[must_use]
    pub const fn min_consecutive(min_size: usize) -> Self {
        Self {
            min_size,
            max_gap: 0,
            allowed_gap_commands: Vec::new(),
        }
    }

    /// Allow gaps of up to `n` commands between matches
    #[must_use]
    pub const fn with_max_gap(mut self, max_gap: usize) -> Self {
        self.max_gap = max_gap;
        self
    }

    /// Allow specific commands to appear in gaps
    #[must_use]
    pub fn with_allowed_gaps(mut self, commands: Vec<&'static str>) -> Self {
        self.allowed_gap_commands = commands;
        self
    }
}

/// A pair of consecutive pipeline elements that match specific commands.
#[derive(Debug)]
pub struct CommandPair<'a> {
    /// First command call
    pub first: &'a Call,
    /// Second command call
    pub second: &'a Call,
    /// Index of first element in pipeline
    pub first_index: usize,
    /// Index of second element in pipeline
    pub second_index: usize,
    /// Combined span
    pub span: Span,
}

pub trait PipelineExt {
    /// Infers parameter type from pipeline. Example: `$text | str length`
    /// infers `string`
    fn infer_param_type(&self, param_var_id: VarId, context: &LintContext) -> Option<Type>;

    /// Find clusters of consecutive calls to a specific command.
    ///
    /// # Example
    /// ```ignore
    /// // Find 2+ consecutive `append` calls
    /// let clusters = pipeline.find_command_clusters("append", ctx, &ClusterConfig::min_consecutive(2));
    /// ```
    fn find_command_clusters<'a>(
        &'a self,
        command_name: &str,
        context: &LintContext,
        config: &ClusterConfig,
    ) -> Vec<CommandCluster<'a>>;

    /// Find pairs of consecutive commands matching specific patterns.
    ///
    /// # Example
    /// ```ignore
    /// // Find `split row | get` patterns
    /// let pairs = pipeline.find_command_pairs(ctx,
    ///     |c, ctx| c.is_call_to_command("split row", ctx),
    ///     |c, ctx| c.is_call_to_command("get", ctx));
    /// ```
    fn find_command_pairs<'a, F1, F2>(
        &'a self,
        context: &LintContext,
        first_predicate: F1,
        second_predicate: F2,
    ) -> Vec<CommandPair<'a>>
    where
        F1: Fn(&Call, &LintContext) -> bool,
        F2: Fn(&Call, &LintContext) -> bool;
}

impl PipelineExt for Pipeline {
    fn infer_param_type(&self, param_var_id: VarId, context: &LintContext) -> Option<Type> {
        log::trace!(
            "infer_param_type from pipeline: param_var_id={:?}, pipeline_elements={}",
            param_var_id,
            self.elements.len()
        );

        let result = self
            .elements
            .windows(2)
            .find_map(|window| infer_from_pipeline_window(param_var_id, window, context));

        log::trace!("infer_param_type from pipeline result: {result:?}");
        result
    }

    fn find_command_clusters<'a>(
        &'a self,
        command_name: &str,
        context: &LintContext,
        config: &ClusterConfig,
    ) -> Vec<CommandCluster<'a>> {
        let matches: Vec<(usize, &Call)> = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(idx, elem)| {
                if let Expr::Call(call) = &elem.expr.expr
                    && call.is_call_to_command(command_name, context)
                {
                    return Some((idx, call.as_ref()));
                }
                None
            })
            .collect();

        if matches.len() < config.min_size {
            return Vec::new();
        }

        build_clusters_from_matches(&matches, self, context, config)
    }

    fn find_command_pairs<'a, F1, F2>(
        &'a self,
        context: &LintContext,
        first_predicate: F1,
        second_predicate: F2,
    ) -> Vec<CommandPair<'a>>
    where
        F1: Fn(&Call, &LintContext) -> bool,
        F2: Fn(&Call, &LintContext) -> bool,
    {
        self.elements
            .windows(2)
            .enumerate()
            .filter_map(|(idx, window)| {
                let (Expr::Call(first), Expr::Call(second)) =
                    (&window[0].expr.expr, &window[1].expr.expr)
                else {
                    return None;
                };

                (first_predicate(first, context) && second_predicate(second, context)).then(|| {
                    CommandPair {
                        first,
                        second,
                        first_index: idx,
                        second_index: idx + 1,
                        span: Span::new(window[0].expr.span.start, window[1].expr.span.end),
                    }
                })
            })
            .collect()
    }
}

// Private helper functions

fn infer_from_pipeline_window(
    param_var_id: VarId,
    window: &[PipelineElement],
    context: &LintContext,
) -> Option<Type> {
    let contains_param = window[0].expr.contains_variable(param_var_id);
    log::trace!(
        "  Checking pipeline window: contains_param={}, first_expr={:?}, second_expr={:?}",
        contains_param,
        &window[0].expr.expr,
        &window[1].expr.expr
    );

    let Expr::Call(call) = &window[1].expr.expr else {
        log::trace!("  -> Not a call expression");
        return None;
    };

    if !contains_param {
        log::trace!("  -> Parameter not used in first element");
        return None;
    }

    let decl = context.working_set.get_decl(call.decl_id);
    let sig = decl.signature();

    let Some((input_type, _)) = sig.input_output_types.first() else {
        log::trace!("  -> No input/output types for '{}'", decl.name());
        return None;
    };

    if matches!(input_type, Type::Any) {
        log::trace!(
            "  -> Found call to '{}', but input_type is Any",
            decl.name()
        );
        return None;
    }

    log::trace!(
        "  -> Found call to '{}', input_type={:?}",
        decl.name(),
        input_type
    );
    Some(input_type.clone())
}

fn build_clusters_from_matches<'a>(
    matches: &[(usize, &'a Call)],
    pipeline: &Pipeline,
    context: &LintContext,
    config: &ClusterConfig,
) -> Vec<CommandCluster<'a>> {
    let mut clusters = Vec::new();
    let mut current_indices = vec![matches[0].0];
    let mut current_calls = vec![matches[0].1];

    for window in matches.windows(2) {
        let (prev_idx, _) = window[0];
        let (curr_idx, curr_call) = window[1];
        let gap = curr_idx - prev_idx - 1;

        let valid_gap = gap == 0
            || (gap <= config.max_gap
                && is_valid_gap(pipeline, prev_idx + 1, curr_idx, context, config));

        if valid_gap {
            current_indices.push(curr_idx);
            current_calls.push(curr_call);
        } else {
            if current_indices.len() >= config.min_size {
                clusters.push(build_cluster(&current_indices, &current_calls, pipeline));
            }
            current_indices = vec![curr_idx];
            current_calls = vec![curr_call];
        }
    }

    if current_indices.len() >= config.min_size {
        clusters.push(build_cluster(&current_indices, &current_calls, pipeline));
    }

    clusters
}

fn is_valid_gap(
    pipeline: &Pipeline,
    start: usize,
    end: usize,
    context: &LintContext,
    config: &ClusterConfig,
) -> bool {
    if config.allowed_gap_commands.is_empty() {
        return false;
    }

    (start..end).all(|i| {
        if let Expr::Call(call) = &pipeline.elements[i].expr.expr {
            let name = call.get_call_name(context);
            config.allowed_gap_commands.iter().any(|&cmd| cmd == name)
        } else {
            false
        }
    })
}

fn build_cluster<'a>(
    indices: &[usize],
    calls: &[&'a Call],
    pipeline: &Pipeline,
) -> CommandCluster<'a> {
    let first_idx = indices[0];
    let last_idx = indices[indices.len() - 1];
    let span = Span::new(
        pipeline.elements[first_idx].expr.span.start,
        pipeline.elements[last_idx].expr.span.end,
    );

    CommandCluster {
        indices: indices.to_vec(),
        calls: calls.to_vec(),
        span,
    }
}
