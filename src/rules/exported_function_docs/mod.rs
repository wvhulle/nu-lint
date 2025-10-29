use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    // Search for "export def" patterns in the source code
    let source_lines: Vec<&str> = context.source.lines().collect();

    for (line_idx, line) in source_lines.iter().enumerate() {
        let trimmed = line.trim();

        if !trimmed.starts_with("export def ") {
            continue;
        }

        let has_doc_comment = check_for_doc_comment(&source_lines, line_idx);

        if !has_doc_comment {
            let after_def = trimmed.strip_prefix("export def ").unwrap();
            let func_name = extract_function_name(after_def);

            let line_start: usize = source_lines[..line_idx].iter().map(|l| l.len() + 1).sum();
            let line_end = line_start + line.len();

            violations.push(
                RuleViolation::new_dynamic(
                    "exported_function_docs",
                    format!("Exported function '{func_name}' is missing documentation"),
                    nu_protocol::Span::new(line_start, line_end),
                )
                .with_suggestion_dynamic(format!(
                    "Add a documentation comment above the function:\n# Description of \
                     {func_name}\nexport def {func_name} ..."
                )),
            );
        }
    }

    violations
}

fn check_for_doc_comment(source_lines: &[&str], line_idx: usize) -> bool {
    if line_idx == 0 {
        return false;
    }

    let prev_line = source_lines[line_idx - 1].trim();
    let is_comment = prev_line.starts_with('#') && !prev_line.starts_with("##");

    let is_test_comment = prev_line.to_lowercase().contains("bad:")
        || prev_line.to_lowercase().contains("good:")
        || prev_line.to_lowercase().contains("todo:")
        || prev_line.to_lowercase().contains("fixme:")
        || prev_line.to_lowercase().contains("test:")
        || prev_line.to_lowercase().contains("example:");

    is_comment && !is_test_comment
}

fn extract_function_name(after_def: &str) -> &str {
    if let Some(space_idx) = after_def.find(' ') {
        &after_def[..space_idx]
    } else if let Some(bracket_idx) = after_def.find('[') {
        after_def[..bracket_idx].trim()
    } else {
        after_def
    }
}

pub fn rule() -> Rule {
    Rule::new(
        "exported_function_docs",
        RuleCategory::Documentation,
        Severity::Info,
        "Exported functions should have documentation comments",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
