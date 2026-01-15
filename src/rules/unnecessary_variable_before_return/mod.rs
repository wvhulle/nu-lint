use nu_protocol::{
    Span, VarId,
    ast::{Block, Call, Expr, Expression, Pipeline},
};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Fix data: spans needed to reconstruct the fix
pub struct FixData {
    /// The span covering both the let statement and the return reference
    combined_span: Span,
    /// The span of the value expression in the let statement
    value_span: Span,
}

struct LetDeclaration<'a> {
    var_id: VarId,
    var_name: String,
    span: Span,
    call: &'a Call,
}

fn extract_let_declaration<'a>(
    expr: &'a Expression,
    context: &LintContext,
) -> Option<LetDeclaration<'a>> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if !call.is_call_to_command("let", context) {
        return None;
    }

    let (var_id, var_name, _var_span) = call.extract_variable_declaration(context)?;
    Some(LetDeclaration {
        var_id,
        var_name,
        span: expr.span,
        call,
    })
}

fn extract_value_from_let(call: &Call) -> Option<&Expression> {
    call.get_positional_arg(1)
}

/// Check if an expression is just a variable reference
fn extract_var_reference(expr: &Expression) -> Option<VarId> {
    match &expr.expr {
        Expr::Var(var_id) | Expr::VarDecl(var_id) => Some(*var_id),
        Expr::FullCellPath(cell_path) if cell_path.tail.is_empty() => {
            // Simple variable without path members (e.g., $var, not $var.field)
            match &cell_path.head.expr {
                Expr::Var(var_id) => Some(*var_id),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Check if a pipeline contains only a single variable reference
fn is_simple_var_reference(pipeline: &Pipeline, var_id: VarId) -> Option<Span> {
    if pipeline.elements.len() != 1 {
        return None;
    }

    let element = pipeline.elements.first()?;
    let referenced_var_id = extract_var_reference(&element.expr)?;

    (referenced_var_id == var_id).then_some(element.expr.span)
}

/// Check a block for the unnecessary variable pattern
fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<(Detection, FixData)>) {
    let pipelines = &block.pipelines;

    for i in 0..pipelines.len().saturating_sub(1) {
        let current_pipeline = &pipelines[i];
        let next_pipeline = &pipelines[i + 1];

        // Check if current pipeline has a single element
        if current_pipeline.elements.len() != 1 {
            continue;
        }

        let element = &current_pipeline.elements[0];

        let Some(let_decl) = extract_let_declaration(&element.expr, context) else {
            continue;
        };

        if let Some(ref_span) = is_simple_var_reference(next_pipeline, let_decl.var_id) {
            log::debug!(
                "Found unnecessary variable pattern: let {} = ... followed by ${}",
                let_decl.var_name,
                let_decl.var_name
            );
            let combined_span = Span::new(let_decl.span.start, ref_span.end);

            let Some(value_expr) = extract_value_from_let(let_decl.call) else {
                continue;
            };

            let violation = Detection::from_global_span(
                format!(
                    "Variable '{}' is assigned and immediately returned - consider returning the \
                     expression directly",
                    let_decl.var_name
                ),
                let_decl.span,
            )
            .with_primary_label("variable declared here")
            .with_extra_label("immediately returned here", ref_span);

            let fix_data = FixData {
                combined_span,
                value_span: value_expr.span,
            };

            violations.push((violation, fix_data));
        }
    }
}

struct UnnecessaryVariableBeforeReturn;

impl DetectFix for UnnecessaryVariableBeforeReturn {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "unnecessary_variable_before_return"
    }

    fn short_description(&self) -> &'static str {
        "Variable assigned and immediately returned adds unnecessary verbosity"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/thinking_in_nu.html#implicit-return")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();

        // Check the main block
        check_block(context.ast, context, &mut violations);

        // Recursively check all nested blocks (closures, functions, etc.)
        violations.extend(context.detect_with_fix_data(|expr, ctx| match &expr.expr {
            Expr::Closure(block_id) | Expr::Block(block_id) => {
                let mut nested_violations = Vec::new();
                let block = ctx.working_set.get_block(*block_id);
                check_block(block, ctx, &mut nested_violations);
                nested_violations
            }
            _ => vec![],
        }));

        violations
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement_text = context.plain_text(fix_data.value_span).to_string();
        Some(Fix::with_explanation(
            format!("Return expression directly: {replacement_text}"),
            vec![Replacement::new(fix_data.combined_span, replacement_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &UnnecessaryVariableBeforeReturn;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
