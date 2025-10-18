use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn check(context: &LintContext) -> Vec<Violation> {
    // Pattern: for item in $collection { ... }
    let for_loop_pattern = Regex::new(r"for\s+(\w+)\s+in\s+(\$\w+|\([^\)]+\))\s*\{").unwrap();

    context.violations_from_regex(
        &for_loop_pattern,
        "prefer_each_over_for",
        Severity::Warning,
        |mat| {
            let caps = for_loop_pattern.captures(mat.as_str())?;
            let item_var = &caps[1];
            let collection = &caps[2];

            // Get a snippet of the loop body to check if it's doing side effects
            let remaining = &context.source[mat.end()..];
            let body_end = remaining.find('}').unwrap_or(100.min(remaining.len()));
            let body = &remaining[..body_end];

            // Enhanced side effect detection
            let has_external_commands = body.contains('^');
            let has_print = body.contains("print");
            let has_mutation = body.starts_with("mut ") || body.contains(" mut ");
            let has_file_ops = body.contains("save")
                || body.contains("rm ")
                || body.contains("mkdir")
                || body.contains("touch")
                || body.contains("cp ")
                || body.contains("mv ");
            let has_system_ops = body.contains("exit") || body.contains("cd ");

            // Check for external tools that might not use ^
            let external_tool_pattern =
                Regex::new(r"\b(ffmpeg|curl|git|docker|ssh|wget|tar|zip|unzip|rsync)\b").unwrap();
            let has_external_tools = external_tool_pattern.is_match(body);

            // Only suggest 'each' if there are no obvious side effects
            let has_side_effects = has_external_commands
                || has_print
                || has_mutation
                || has_file_ops
                || has_system_ops
                || has_external_tools;

            if has_side_effects {
                None
            } else {
                Some((
                    format!(
                        "For loop iterating '{item_var}' - consider using 'each' for functional \
                         style"
                    ),
                    Some(format!(
                        "Use '{collection} | each {{ |{item_var}| ... }}' for functional iteration"
                    )),
                ))
            }
        },
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_each_over_for",
        RuleCategory::Idioms,
        Severity::Warning,
        "Use 'each' pipeline instead of 'for' loops for functional style",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
