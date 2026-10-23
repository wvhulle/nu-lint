use std::fmt::Write;

use nu_lint_analysis::fix::FixResult;

use super::format_diff_context;

#[must_use]
pub fn format_fix_results(results: &[FixResult], dry_run: bool) -> String {
    let mut output = String::new();

    if results.is_empty() {
        output.push_str("No fixable violations found.\n");
        return output;
    }

    if dry_run {
        writeln!(
            output,
            "The following changes would be applied ({} file{}):\n",
            results.len(),
            if results.len() == 1 { "" } else { "s" }
        )
        .unwrap();

        for result in results {
            writeln!(output, "File: {}", result.file_path.display()).unwrap();
            writeln!(output, "Fixes to apply: {}\n", result.fixes_applied).unwrap();

            let diff = format_diff_context(&result.original_content, &result.fixed_content);
            output.push_str(&diff);
            output.push('\n');
        }
    } else {
        writeln!(
            output,
            "Fixed {} file{}:\n",
            results.len(),
            if results.len() == 1 { "" } else { "s" }
        )
        .unwrap();

        for result in results {
            writeln!(
                output,
                "  {} ({} fix{})",
                result.file_path.display(),
                result.fixes_applied,
                if result.fixes_applied == 1 { "" } else { "es" }
            )
            .unwrap();
        }
    }

    output
}
