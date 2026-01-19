use lsp_types::{DiagnosticTag, Hover, HoverContents, MarkupContent, MarkupKind};

use crate::{config::LintLevel, rules::groups::groups_for_rule, violation::Violation};

/// Convert a violation to hover markdown documentation.
pub fn violation_to_hover_markdown(v: &Violation) -> String {
    let mut lines = Vec::new();

    // Header with rule ID and category badges
    if let Some(rule_id) = v.rule_id.as_deref() {
        let categories = groups_for_rule(rule_id);
        let badges = format!("({})", categories.join(", "));
        lines.push(format!("### `{rule_id} `{badges}"));
    }

    // Short description as subtitle
    if let Some(short) = v.short_description {
        lines.push(format!("*{short}*"));
    }

    // Long description
    if let Some(desc) = &v.long_description {
        lines.push(String::new());
        lines.push(desc.clone());
    }

    // Status badges (lint level + diagnostic tags)
    let mut badges = Vec::new();
    match v.lint_level {
        LintLevel::Error => badges.push("Error"),
        LintLevel::Warning => badges.push("Warning"),
        LintLevel::Hint => badges.push("Hint"),
    }
    for tag in &v.diagnostic_tags {
        match *tag {
            DiagnosticTag::UNNECESSARY => badges.push("Unnecessary"),
            DiagnosticTag::DEPRECATED => badges.push("Deprecated"),
            _ => {}
        }
    }

    // Documentation link
    if let Some(url) = v.doc_url {
        lines.push(String::new());
        lines.push(format!("[{url}]({url})"));
    }

    lines.join("\n")
}

/// Build hover documentation from violations at a position.
pub fn build_hover<'a>(violations: impl Iterator<Item = &'a Violation>) -> Option<Hover> {
    let markdown: String = violations
        .map(violation_to_hover_markdown)
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    (!markdown.is_empty()).then_some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown,
        }),
        range: None,
    })
}
