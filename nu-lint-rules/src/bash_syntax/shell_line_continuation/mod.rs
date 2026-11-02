use nu_lint_analysis::{
    LintLevel,
    ast::lexed_statement::{
        LexedStatement, all_lexed_statements, token_text_equals, user_file_span,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::Span;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

fn next_line_content_start(backslash_end: usize, lint_context: &LintContext) -> Option<usize> {
    let file_end = user_file_span(lint_context).end;
    let following_bytes = lint_context
        .working_set
        .get_span_contents(Span::new(backslash_end, file_end));
    let newline_width = match following_bytes {
        [b'\n', ..] => 1,
        [b'\r', b'\n', ..] => 2,
        _ => return None,
    };
    following_bytes[newline_width..]
        .iter()
        .position(|following_byte| !following_byte.is_ascii_whitespace())
        .map(|indent_width| backslash_end + newline_width + indent_width)
}

fn continuation_span(lexed_statement: &LexedStatement, lint_context: &LintContext) -> Option<Span> {
    let [.., previous_token, backslash_token] = lexed_statement.statement_tokens.as_slice() else {
        return None;
    };
    token_text_equals(backslash_token, lint_context, b"\\")
        .then(|| next_line_content_start(backslash_token.span.end, lint_context))
        .flatten()
        .map(|content_start| Span::new(previous_token.span.end, content_start))
}

struct LineContinuationJoin {
    joined_span: Span,
}

struct ShellLineContinuation;

impl DetectFix for ShellLineContinuation {
    type FixInput<'a> = LineContinuationJoin;

    fn id(&self) -> &'static str {
        "shell_line_continuation"
    }

    fn short_description(&self) -> &'static str {
        "Trailing backslash does not continue the command on the next line"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "A trailing `\\` reaches the command as a literal argument and the next line runs as \
             a separate command, without any error.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/coming_from_bash.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        all_lexed_statements(context)
            .iter()
            .filter_map(|lexed_statement| continuation_span(lexed_statement, context))
            .map(|joined_span| {
                (
                    Detection::from_global_span(
                        "Trailing `\\` is passed to the command as an argument",
                        joined_span,
                    )
                    .with_primary_label("bash line continuation"),
                    LineContinuationJoin { joined_span },
                )
            })
            .collect()
    }

    fn fix(
        &self,
        _lint_context: &LintContext,
        continuation_join: &Self::FixInput<'_>,
    ) -> Option<Fix> {
        Some(Fix {
            explanation: "Join the lines into one command".into(),
            replacements: vec![Replacement::new(
                continuation_join.joined_span,
                " ".to_string(),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &ShellLineContinuation;
