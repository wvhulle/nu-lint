use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};

use crate::{rules::groups::groups_for_rule, violation::Violation};

fn violation_to_hover_markdown(v: &Violation) -> String {
    let mut lines = Vec::new();

    if let Some(rule_id) = v.rule_id.as_deref() {
        let groups = groups_for_rule(rule_id).join(", ");
        lines.push(format!("### `{rule_id}` ({groups})"));
    }

    if let Some(short) = v.short_description {
        lines.push(format!("*{short}*"));
    }

    if let Some(desc) = &v.long_description {
        lines.push(String::new());
        lines.push(desc.clone());
    }

    if let Some(url) = v.doc_url {
        lines.push(String::new());
        lines.push(format!("[Documentation]({url})"));
    }

    lines.join("\n")
}

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
