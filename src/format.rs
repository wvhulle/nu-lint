use std::fmt::Write;

use owo_colors::OwoColorize;

/// Format a diff between old and new text for inline display (violation
/// previews). Shows all removed lines in red, then all added lines in green.
#[must_use]
pub fn format_diff_inline(old_text: &str, new_text: &str) -> String {
    if old_text == new_text {
        return String::new();
    }

    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let format_removed = |line: &str| format!("  - {}", line.red());
    let format_added = |line: &str| format!("  + {}", line.green());

    if old_lines.len() > 1 || new_lines.len() > 1 {
        let removed: String = old_lines
            .iter()
            .map(|l| format_removed(l))
            .collect::<Vec<_>>()
            .join("\n");
        let added: String = new_lines
            .iter()
            .map(|l| format_added(l))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{removed}\n{added}")
    } else {
        format!(
            "{}\n{}",
            format_removed(old_text.trim()),
            format_added(new_text.trim())
        )
    }
}

/// Format a diff between old and new text with line numbers (fix application
/// output). Shows only changed lines with their line numbers.
#[must_use]
pub fn format_diff_context(original: &str, fixed: &str) -> String {
    let original_lines: Vec<&str> = original.lines().collect();
    let fixed_lines: Vec<&str> = fixed.lines().collect();

    if original_lines == fixed_lines {
        return String::from("No changes\n");
    }

    let mut output = String::new();
    let max_lines = original_lines.len().max(fixed_lines.len());

    for i in 0..max_lines {
        let orig = original_lines.get(i);
        let fix = fixed_lines.get(i);

        match (orig, fix) {
            (Some(o), Some(f)) if o != f => {
                writeln!(output, "{}", format!("-{:>4} {o}", i + 1).red()).unwrap();
                writeln!(output, "{}", format!("+{:>4} {f}", i + 1).green()).unwrap();
            }
            (Some(o), None) => {
                writeln!(output, "{}", format!("-{:>4} {o}", i + 1).red()).unwrap();
            }
            (None, Some(f)) => {
                writeln!(output, "{}", format!("+{:>4} {f}", i + 1).green()).unwrap();
            }
            _ => {}
        }
    }

    if output.is_empty() {
        String::from("No changes\n")
    } else {
        output
    }
}

/// Format a URL as a clickable terminal hyperlink (OSC 8 escape sequence).
#[must_use]
pub fn format_clickable_url(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
}
