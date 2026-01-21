use lsp_types::DiagnosticTag;
use nu_protocol::{
    DeclId, Span, VarId,
    ast::{self, Argument, Expr, Traverse},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    rules::typing::{format_flag, format_optional, format_required, format_rest},
    violation::{Detection, Fix, Replacement},
};

#[derive(Clone)]
enum ParamType {
    Required(usize),
    Optional(usize),
    Rest(usize),
    Named(String),
}

struct FixData {
    param_name: String,
    name_span: Span,
    signature_span: Span,
    decl_id: Option<DeclId>,
    is_exported: bool,
    param_type: ParamType,
    signature: nu_protocol::Signature,
}

struct UnusedParameter;

impl DetectFix for UnusedParameter {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "unused_parameter"
    }

    fn short_description(&self) -> &'static str {
        "Function parameter declared but never used"
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::UNNECESSARY]
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, context| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            let Some(def) = call.custom_command_def(context) else {
                return vec![];
            };

            let block = context.working_set.get_block(def.body);
            let check_ctx = CheckContext {
                block,
                func_name: &def.name,
                context,
                signature_span: def.signature_span,
                decl_id: context.working_set.find_decl(def.name.as_bytes()),
                is_exported: def.is_exported() || def.is_main(),
                signature: def.signature.clone(),
            };

            let required_count = def.signature.required_positional.len();
            let rest_start_idx = required_count + def.signature.optional_positional.len();

            def.signature
                .required_positional
                .iter()
                .enumerate()
                .filter_map(|(idx, p)| {
                    check_parameter(&p.name, p.var_id, ParamType::Required(idx), &check_ctx)
                })
                .chain(
                    def.signature
                        .optional_positional
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, p)| {
                            check_parameter(
                                &p.name,
                                p.var_id,
                                ParamType::Optional(required_count + idx),
                                &check_ctx,
                            )
                        }),
                )
                .chain(def.signature.rest_positional.iter().filter_map(|p| {
                    check_parameter(
                        &p.name,
                        p.var_id,
                        ParamType::Rest(rest_start_idx),
                        &check_ctx,
                    )
                }))
                .chain(def.signature.named.iter().filter_map(|f| {
                    check_parameter(
                        &f.long,
                        f.var_id,
                        ParamType::Named(f.long.clone()),
                        &check_ctx,
                    )
                }))
                .collect()
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // For exported functions or if we can't resolve decl_id, just prefix with
        // underscore
        if fix_data.is_exported || fix_data.decl_id.is_none() {
            let new_name = format!("_{}", fix_data.param_name);
            return Some(Fix {
                explanation: format!("Prefix `{}` with underscore", fix_data.param_name).into(),
                replacements: vec![Replacement::new(fix_data.name_span, new_name)],
            });
        }

        // For non-exported functions, remove the parameter and update call sites
        let mut replacements = vec![remove_param_from_signature(fix_data)];
        replacements.extend(find_call_site_removals(fix_data, context));

        Some(Fix {
            explanation: format!("Remove unused parameter '{}'", fix_data.param_name).into(),
            replacements,
        })
    }
}

struct CheckContext<'a> {
    block: &'a ast::Block,
    func_name: &'a str,
    context: &'a LintContext<'a>,
    signature_span: Span,
    decl_id: Option<DeclId>,
    is_exported: bool,
    signature: nu_protocol::Signature,
}

fn check_parameter(
    param_name: &str,
    var_id: Option<VarId>,
    param_type: ParamType,
    ctx: &CheckContext<'_>,
) -> Option<(Detection, FixData)> {
    // Skip underscore-prefixed parameters (intentionally unused)
    if param_name.starts_with('_') {
        return None;
    }

    // Skip the built-in --help flag
    if let ParamType::Named(ref name) = param_type
        && name == "help"
    {
        return None;
    }

    let var_id = var_id?;

    // Check if parameter is used in the function body
    let usages = ctx.block.var_usages(var_id, ctx.context);

    if !usages.is_empty() {
        return None;
    }

    // Get the parameter's declaration span
    let var = ctx.context.working_set.get_variable(var_id);
    let param_span = var.declaration_span;

    let detection = Detection::from_global_span(
        format!(
            "Parameter '{param_name}' in function '{}' is never used",
            ctx.func_name
        ),
        param_span,
    )
    .with_primary_label("unused parameter");

    Some((
        detection,
        FixData {
            param_name: param_name.to_string(),
            name_span: param_span,
            signature_span: ctx.signature_span,
            decl_id: ctx.decl_id,
            is_exported: ctx.is_exported,
            param_type,
            signature: ctx.signature.clone(),
        },
    ))
}

/// Generate new signature text with the specified parameter removed
fn remove_param_from_signature(fix_data: &FixData) -> Replacement {
    let sig = &fix_data.signature;

    let parts: Vec<String> = sig
        .required_positional
        .iter()
        .enumerate()
        .filter(|(idx, _)| !matches!(fix_data.param_type, ParamType::Required(i) if i == *idx))
        .map(|(_, p)| format_required(p))
        .chain(
            sig.optional_positional
                .iter()
                .enumerate()
                .filter(|(idx, _)| {
                    let full_idx = sig.required_positional.len() + idx;
                    !matches!(fix_data.param_type, ParamType::Optional(i) if i == full_idx)
                })
                .map(|(_, p)| format_optional(p)),
        )
        .chain(
            sig.rest_positional
                .iter()
                .filter(|_| !matches!(fix_data.param_type, ParamType::Rest(_)))
                .map(format_rest),
        )
        .chain(
            sig.named
                .iter()
                .filter(|f| f.long != "help")
                .filter(
                    |f| !matches!(&fix_data.param_type, ParamType::Named(name) if name == &f.long),
                )
                .map(format_flag),
        )
        .collect();

    let new_sig = format!("[{}]", parts.join(", "));
    Replacement::new(fix_data.signature_span, new_sig)
}

/// Find all call sites and generate replacements to remove the corresponding
/// argument
fn find_call_site_removals(fix_data: &FixData, ctx: &LintContext) -> Vec<Replacement> {
    let decl_id = fix_data.decl_id.unwrap();
    let mut replacements = Vec::new();

    ctx.ast.flat_map(
        ctx.working_set,
        &|expr| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };
            if call.decl_id != decl_id {
                return vec![];
            }

            match &fix_data.param_type {
                ParamType::Required(idx) | ParamType::Optional(idx) => {
                    remove_positional_arg(call, *idx, ctx)
                }
                ParamType::Named(flag_name) => remove_named_arg(call, flag_name, ctx),
                ParamType::Rest(start_idx) => {
                    remove_trailing_positional_args(call, *start_idx, ctx)
                }
            }
        },
        &mut replacements,
    );

    replacements
}

/// Remove the nth positional argument from a call
fn remove_positional_arg(
    call: &ast::Call,
    target_idx: usize,
    ctx: &LintContext,
) -> Vec<Replacement> {
    // Collect all positional arguments with their indices
    let positionals: Vec<_> = call
        .arguments
        .iter()
        .filter_map(|arg| match arg {
            Argument::Positional(e) | Argument::Unknown(e) => Some(e),
            _ => None,
        })
        .collect();

    if target_idx >= positionals.len() {
        return vec![];
    }

    let target_span = positionals[target_idx].span;

    // Determine the removal span (include leading/trailing whitespace/comma)
    let removal_span = calculate_removal_span(target_span, target_idx, &positionals, ctx);

    vec![Replacement::new(removal_span, String::new())]
}

/// Remove a named flag and its value from a call
fn remove_named_arg(call: &ast::Call, flag_name: &str, _ctx: &LintContext) -> Vec<Replacement> {
    for arg in &call.arguments {
        if let Argument::Named((name, _, value)) = arg
            && name.item == flag_name
        {
            // Calculate span: from flag name to end of value (if any)
            let start = name.span.start;
            let end = value.as_ref().map_or(name.span.end, |v| v.span.end);
            let span = Span::new(start, end);
            return vec![Replacement::new(span, String::new())];
        }
    }
    vec![]
}

/// Remove all positional arguments from `start_idx` onwards
fn remove_trailing_positional_args(
    call: &ast::Call,
    start_idx: usize,
    _ctx: &LintContext,
) -> Vec<Replacement> {
    let positionals: Vec<_> = call
        .arguments
        .iter()
        .filter_map(|arg| match arg {
            Argument::Positional(e) | Argument::Unknown(e) => Some(e.span),
            _ => None,
        })
        .collect();

    if start_idx >= positionals.len() {
        return vec![];
    }

    // Remove all args from start_idx to end
    let start = positionals[start_idx].start;
    let end = positionals.last().unwrap().end;
    let span = Span::new(start, end);

    vec![Replacement::new(span, String::new())]
}

/// Calculate the span to remove, including appropriate whitespace
fn calculate_removal_span(
    target_span: Span,
    target_idx: usize,
    positionals: &[&ast::Expression],
    ctx: &LintContext,
) -> Span {
    let is_last = target_idx == positionals.len() - 1;
    let is_first = target_idx == 0;

    if positionals.len() == 1 {
        // Only argument - just remove it
        return target_span;
    }

    if is_last {
        // Last argument - extend start to include preceding whitespace
        let prev_end = positionals[target_idx - 1].span.end;
        Span::new(prev_end, target_span.end)
    } else if is_first {
        // First argument - extend end to include trailing whitespace
        let next_start = positionals[target_idx + 1].span.start;
        Span::new(target_span.start, next_start)
    } else {
        // Middle argument - extend start to include preceding whitespace
        let prev_end = positionals[target_idx - 1].span.end;
        // Check if there's meaningful content between prev and target
        let between = ctx.span_text(Span::new(prev_end, target_span.start));
        if between.trim().is_empty() || between.trim() == "," {
            Span::new(prev_end, target_span.end)
        } else {
            target_span
        }
    }
}

pub static RULE: &dyn Rule = &UnusedParameter;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
