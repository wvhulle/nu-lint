use nu_protocol::ast::{Expr, Pipeline};

use crate::{Fix, Replacement, context::LintContext, rule::Rule, violation::Violation};
/// Check if a pipeline starts with redundant $in
fn pipeline_starts_with_redundant_in(pipeline: &Pipeline, context: &LintContext) -> bool {
    log::debug!(
        "Checking pipeline with {} elements",
        pipeline.elements.len()
    );
    // Only check if this pipeline has exactly one element and that element is a
    // Collect This ensures we're only looking at cases like "{ $in | command }"
    // as the main function body, not cases where $in is used within larger
    // expressions
    if pipeline.elements.len() != 1 {
        log::debug!("Pipeline doesn't have exactly 1 element, skipping");
        return false;
    }
    let element = &pipeline.elements[0];
    log::debug!("Single element: {:?}", element.expr.expr);
    // Check if this element is a Collect expression (which represents $in | ...)
    if let Expr::Collect(_, inner_expr) = &element.expr.expr {
        log::debug!("Found Collect expression (representing $in | ...)");
        // Look at the inner expression to see if it's a subexpression (pipeline)
        if let Expr::Subexpression(block_id) = &inner_expr.expr {
            log::debug!("Collect contains subexpression with block {block_id:?}");
            let inner_block = context.working_set.get_block(*block_id);
            log::debug!("Inner block has {} pipelines", inner_block.pipelines.len());
            // Check if this looks like a simple pipeline operation that could be simplified
            // Only flag cases where the inner block is a straightforward pipeline
            if inner_block.pipelines.len() != 1 {
                log::debug!("Inner block doesn't have exactly 1 pipeline");
                return false;
            }
            let inner_pipeline = &inner_block.pipelines[0];
            log::debug!(
                "Single inner pipeline with {} elements",
                inner_pipeline.elements.len()
            );
            // Check if this is a simple pipeline like "$in | command" or "$in | command1 |
            // command2" These are cases where the $in can be omitted
            if inner_pipeline.elements.len() < 2 {
                log::debug!("Pipeline too short");
                return false;
            }
            // Look at the first element - it should be a variable reference to $in
            let Some(first_element) = inner_pipeline.elements.first() else {
                log::debug!("No first element in pipeline");
                return false;
            };
            log::debug!("First inner element: {:?}", first_element.expr.expr);
            // If the first element is a Var (referring to $in) followed by commands,
            // this is the pattern we want to detect
            if matches!(
                &first_element.expr.expr,
                Expr::Var(_) | Expr::FullCellPath(_)
            ) {
                log::debug!(
                    "Found $in variable followed by pipeline operations - this is redundant"
                );
                return true;
            }
            log::debug!("Inner block doesn't match simple redundant $in pattern");
            return false;
        }
    }
    log::debug!("No redundant $in found in this pipeline");
    false
}
fn extract_function_body_from_source(decl_name: &str, context: &LintContext) -> Option<String> {
    let decl_span = context.find_declaration_span(decl_name);
    let contents = String::from_utf8_lossy(context.working_set.get_span_contents(decl_span));
    log::debug!(
        "Extracting body for '{decl_name}' from source: span={decl_span:?}, contents='{contents}'"
    );
    contents
        .find('{')
        .and_then(|start| contents.rfind('}').map(|end| (start, end)))
        .map(|(start, end)| {
            let body = contents[start + 1..end].trim().to_string();
            log::debug!("Extracted body: '{body}'");
            body
        })
}
fn extract_function_body_from_block(
    block_span: Option<nu_protocol::Span>,
    context: &LintContext,
) -> Option<String> {
    block_span.and_then(|span| {
        let contents = String::from_utf8_lossy(context.working_set.get_span_contents(span));
        let trimmed = contents.trim();
        trimmed
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
            .map(|stripped| stripped.trim().to_string())
    })
}
fn remove_redundant_in_from_body(function_body: &str) -> String {
    let trimmed = function_body.trim_start();
    if trimmed.starts_with("$in | ") {
        function_body.replacen("$in | ", "", 1)
    } else if trimmed.starts_with("$in|") {
        function_body.replacen("$in|", "", 1)
    } else {
        function_body.replace("$in | ", "").replace("$in|", "")
    }
}
fn format_parameters(signature: &nu_protocol::Signature) -> String {
    signature
        .required_positional
        .iter()
        .chain(signature.optional_positional.iter())
        .map(|p| p.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
fn generate_fix_text(
    signature: &nu_protocol::Signature,
    block_span: Option<nu_protocol::Span>,
    context: &LintContext,
) -> Option<String> {
    let body = extract_function_body_from_block(block_span, context)
        .or_else(|| extract_function_body_from_source(&signature.name, context))?;
    let transformed = remove_redundant_in_from_body(&body);
    Some(format!(
        "def {} [{}] {{ {} }}",
        signature.name,
        format_parameters(signature),
        transformed.trim()
    ))
}
fn create_fix(fix_text: String, span: nu_protocol::Span) -> Fix {
    Fix::new_dynamic(
        fix_text.clone(),
        vec![Replacement::new_dynamic(span, fix_text)],
    )
}
fn create_violation(
    signature: &nu_protocol::Signature,
    block_span: Option<nu_protocol::Span>,
    context: &LintContext,
) -> Violation {
    let span = context.find_declaration_span(&signature.name);
    let suggestion = "Remove redundant $in - it's implicit at the start of pipelines";
    let violation = Violation::new_dynamic(
        "remove_redundant_in",
        format!(
            "Redundant $in usage in function '{}' - $in is implicit at the start of pipelines",
            signature.name
        ),
        span,
    )
    .with_suggestion_dynamic(suggestion.to_string());
    generate_fix_text(signature, block_span, context).map_or(violation.clone(), |fix_text| {
        violation.with_fix(create_fix(fix_text, span))
    })
}
fn check(context: &LintContext) -> Vec<Violation> {
    let user_functions: Vec<_> = context.new_user_functions().collect();
    log::debug!(
        "remove_redundant_in: Found {} user functions",
        user_functions.len()
    );
    user_functions
        .into_iter()
        .filter_map(|(_, decl)| {
            let signature = decl.signature();
            log::debug!("Checking function: {}", signature.name);
            // Check if the function body starts with redundant $in
            // Only check the top-level function body, not nested blocks (like if/else
            // branches)
            let Some(block_id) = decl.block_id() else {
                log::debug!("Function {} has no block", signature.name);
                return None;
            };
            let block = context.working_set.get_block(block_id);
            log::debug!(
                "Function {} has {} pipelines",
                signature.name,
                block.pipelines.len()
            );
            let mut has_redundant_in = false;
            for (i, pipeline) in block.pipelines.iter().enumerate() {
                log::debug!("Checking top-level pipeline {i}");
                if pipeline_starts_with_redundant_in(pipeline, context) {
                    log::debug!("Found redundant $in in top-level pipeline {i}");
                    has_redundant_in = true;
                    break;
                }
            }
            if !has_redundant_in {
                log::debug!("Function {} does not have redundant $in", signature.name);
                return None;
            }
            Some(create_violation(&signature, block.span, context))
        })
        .collect()
}
pub fn rule() -> Rule {
    Rule::new(
        "remove_redundant_in",
        "Remove redundant $in at the start of pipelines - it's implicit in Nushell",
        check,
    )
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
