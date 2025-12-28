use nu_protocol::ast::{Expr, Pipeline};

use crate::{
    Fix, LintLevel, Replacement,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct FixData {
    fix_span: nu_protocol::Span,
    function_name: String,
    block_span: Option<nu_protocol::Span>,
    signature_params: Vec<String>,
}

/// Check if a pipeline starts with redundant `$in | ...` pattern
fn pipeline_starts_with_redundant_in(pipeline: &Pipeline, context: &LintContext) -> bool {
    let [element] = pipeline.elements.as_slice() else {
        return false;
    };

    let Expr::Collect(_, inner_expr) = &element.expr.expr else {
        return false;
    };

    let Expr::Subexpression(block_id) = &inner_expr.expr else {
        return false;
    };

    let inner_block = context.working_set.get_block(*block_id);
    let [inner_pipeline] = inner_block.pipelines.as_slice() else {
        return false;
    };

    // Need at least 2 elements: `$in | command`
    let Some(first) = inner_pipeline.elements.first() else {
        return false;
    };

    inner_pipeline.elements.len() >= 2
        && matches!(&first.expr.expr, Expr::Var(_) | Expr::FullCellPath(_))
}

fn extract_function_body(
    block_span: Option<nu_protocol::Span>,
    decl_name: &str,
    context: &LintContext,
) -> Option<String> {
    // Try from block span first, then fall back to source extraction
    if let Some(span) = block_span {
        let contents = context.get_span_text(span).trim();
        if let Some(inner) = contents.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            return Some(inner.trim().to_string());
        }
    }

    // Fall back to source extraction
    let span: nu_protocol::Span = context.find_declaration_span(decl_name).into();
    let contents = context.get_span_text(span);
    let start = contents.find('{')?;
    let end = contents.rfind('}')?;
    Some(contents[start + 1..end].trim().to_string())
}

fn remove_redundant_in_from_body(body: &str) -> String {
    let trimmed = body.trim_start();
    if trimmed.starts_with("$in | ") {
        body.replacen("$in | ", "", 1)
    } else if trimmed.starts_with("$in|") {
        body.replacen("$in|", "", 1)
    } else {
        body.replace("$in | ", "").replace("$in|", "")
    }
}

fn extract_signature_params(signature: &nu_protocol::Signature) -> Vec<String> {
    signature
        .required_positional
        .iter()
        .chain(signature.optional_positional.iter())
        .map(|p| p.name.clone())
        .collect()
}

struct RemoveRedundantIn;

impl DetectFix for RemoveRedundantIn {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "remove_redundant_in"
    }

    fn explanation(&self) -> &'static str {
        "Remove redundant $in at the start of pipelines - it's implicit in Nushell"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/special_variables.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .new_user_functions()
            .filter_map(|(_, decl)| {
                let signature = decl.signature();
                let block_id = decl.block_id()?;
                let block = context.working_set.get_block(block_id);

                let has_redundant_in = block
                    .pipelines
                    .iter()
                    .any(|p| pipeline_starts_with_redundant_in(p, context));

                if !has_redundant_in {
                    return None;
                }

                let name_span = context.find_declaration_span(&signature.name);
                let fix_span: nu_protocol::Span = name_span.into();

                let mut violation = Detection::from_file_span(
                    format!("Redundant $in in function '{}'", signature.name),
                    name_span,
                )
                .with_primary_label("function with redundant $in")
                .with_help("Remove redundant $in - it's implicit at the start of pipelines");

                if let Some(body_span) = block.span {
                    violation = violation.with_extra_label("$in used at pipeline start", body_span);
                }

                let fix_data = FixData {
                    fix_span,
                    function_name: signature.name.clone(),
                    block_span: block.span,
                    signature_params: extract_signature_params(&signature),
                };

                Some((violation, fix_data))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let body = extract_function_body(fix_data.block_span, &fix_data.function_name, context)?;
        let transformed = remove_redundant_in_from_body(&body);
        let params = fix_data.signature_params.join(", ");
        let fix_text = format!(
            "def {} [{}] {{ {} }}",
            fix_data.function_name,
            params,
            transformed.trim()
        );

        Some(Fix::with_explanation(
            fix_text.clone(),
            vec![Replacement::new(fix_data.fix_span, fix_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &RemoveRedundantIn;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
