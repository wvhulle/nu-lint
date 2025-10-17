use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;

pub struct PreferEachOverFor;

impl PreferEachOverFor {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferEachOverFor {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferEachOverFor {
    fn id(&self) -> &'static str {
        "prefer_each_over_for"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Use 'each' pipeline instead of 'for' loops for functional style"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Pattern: for item in $collection { ... }
        let for_loop_pattern = Regex::new(r"for\s+(\w+)\s+in\s+(\$\w+|\([^\)]+\))\s*\{").unwrap();

        context.violations_from_regex_if(&for_loop_pattern, self.id(), self.severity(), |mat| {
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
                        "For loop iterating '{item_var}' - consider using 'each' for functional style"
                    ),
                    Some(format!(
                        "Use '{collection} | each {{ |{item_var}| ... }}' for functional iteration"
                    )),
                ))
            }
        })
    }
}


#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
#[cfg(test)]
mod generated_fix;