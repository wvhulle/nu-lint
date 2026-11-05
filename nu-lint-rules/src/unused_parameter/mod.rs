use nu_lint_analysis::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, signature_layout::SignatureLayout},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, DiagnosticTag, Fix, Replacement},
};
use nu_protocol::{
    DeclId, Span, VarId,
    ast::{self, Argument, Expr, Traverse},
};

#[derive(Clone)]
enum ParamType {
    Required(usize),
    Optional(usize),
    Rest(usize),
    Named(usize),
}

struct FixData {
    var_id: VarId,
    name_span: Span,
    signature_span: Span,
    decl_id: Option<DeclId>,
    is_exported: bool,
    param_type: ParamType,
    signature: nu_protocol::Signature,
}

impl FixData {
    fn param_name(&self) -> Option<&str> {
        let signature = &self.signature;
        match self.param_type {
            ParamType::Required(index) => signature
                .required_positional
                .get(index)
                .map(|param| param.name.as_str()),
            ParamType::Optional(index) => signature
                .optional_positional
                .get(index.checked_sub(signature.required_positional.len())?)
                .map(|param| param.name.as_str()),
            ParamType::Rest(_) => signature
                .rest_positional
                .as_ref()
                .map(|param| param.name.as_str()),
            ParamType::Named(index) => signature.named.get(index).map(|flag| flag.long.as_str()),
        }
    }
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

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn diagnostic_tags(&self) -> &'static [DiagnosticTag] {
        &[DiagnosticTag::Unnecessary]
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
                .chain(
                    def.signature
                        .named
                        .iter()
                        .enumerate()
                        .filter_map(|(flag_index, flag)| {
                            check_parameter(
                                &flag.long,
                                flag.var_id,
                                ParamType::Named(flag_index),
                                &check_ctx,
                            )
                        }),
                )
                .collect()
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        // For exported functions or if we can't resolve decl_id, just prefix with
        // underscore
        if fix_data.is_exported || fix_data.decl_id.is_none() {
            let param_name = fix_data.param_name()?;
            let new_name = format!("_{param_name}");
            return Some(Fix {
                explanation: format!("Prefix `{param_name}` with underscore").into(),
                replacements: vec![Replacement::new(fix_data.name_span, new_name)],
            });
        }

        // For non-exported functions, remove the parameter and update call sites
        let mut replacements = vec![Replacement::new(
            SignatureLayout::parse(fix_data.signature_span, context)?
                .removal_span(fix_data.var_id, context)?,
            String::new(),
        )];
        replacements.extend(find_call_site_removals(fix_data, context));

        Some(Fix {
            explanation: format!("Remove unused parameter '{}'", fix_data.param_name()?).into(),
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
    if matches!(param_type, ParamType::Named(_)) && param_name == "help" {
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
            var_id,
            name_span: param_span,
            signature_span: ctx.signature_span,
            decl_id: ctx.decl_id,
            is_exported: ctx.is_exported,
            param_type,
            signature: ctx.signature.clone(),
        },
    ))
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
                    remove_positional_arg(call, *idx)
                }
                ParamType::Named(flag_index) => fix_data
                    .signature
                    .named
                    .get(*flag_index)
                    .map_or_else(Vec::new, |flag| remove_named_arg(call, &flag.long)),
                ParamType::Rest(start_idx) => remove_trailing_positional_args(call, *start_idx),
            }
        },
        &mut replacements,
    );

    replacements
}

fn positional_argument_spans(call: &ast::Call) -> Vec<Span> {
    call.arguments
        .iter()
        .filter_map(|arg| match arg {
            Argument::Positional(e) | Argument::Unknown(e) => Some(e.span),
            _ => None,
        })
        .collect()
}

fn removal_with_preceding_gap(call: &ast::Call, removed: Span) -> Replacement {
    let gap_start = call
        .arguments
        .iter()
        .map(Argument::span)
        .filter(|argument_span| argument_span.end <= removed.start)
        .map(|argument_span| argument_span.end)
        .max()
        .unwrap_or(call.head.end);
    Replacement::new(Span::new(gap_start, removed.end), String::new())
}

fn remove_positional_arg(call: &ast::Call, target_idx: usize) -> Vec<Replacement> {
    positional_argument_spans(call)
        .get(target_idx)
        .map(|target_span| removal_with_preceding_gap(call, *target_span))
        .into_iter()
        .collect()
}

fn remove_named_arg(call: &ast::Call, flag_name: &str) -> Vec<Replacement> {
    call.arguments
        .iter()
        .find(|arg| matches!(arg, Argument::Named((name, _, _)) if name.item == flag_name))
        .map(|arg| removal_with_preceding_gap(call, arg.span()))
        .into_iter()
        .collect()
}

fn remove_trailing_positional_args(call: &ast::Call, start_idx: usize) -> Vec<Replacement> {
    let spans = positional_argument_spans(call);
    let (Some(first_removed), Some(last_removed)) = (spans.get(start_idx), spans.last()) else {
        return vec![];
    };
    vec![removal_with_preceding_gap(
        call,
        Span::new(first_removed.start, last_removed.end),
    )]
}

pub static RULE: &dyn Rule = &UnusedParameter;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
