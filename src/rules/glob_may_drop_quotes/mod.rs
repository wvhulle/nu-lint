use nu_protocol::{
    Span,
    ast::{Argument, Call, Expr},
};

use crate::{
    LintLevel,
    ast::string::bare_glob_needs_quotes,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    quoted_span: Span,
    pattern: String,
}

/// Checks if a string contains glob metacharacters
fn has_glob_chars(content: &str) -> bool {
    content.contains('*') || content.contains('?')
}

/// Checks a single call expression for quoted glob patterns
fn check_call(call: &Call, ctx: &LintContext) -> Vec<(Detection, FixData)> {
    let mut results = vec![];

    for arg in &call.arguments {
        let (Argument::Positional(arg_expr) | Argument::Unknown(arg_expr)) = arg else {
            continue;
        };

        // Check if this is a quoted glob pattern (GlobPattern with is_quoted=true)
        let Expr::GlobPattern(pattern, is_quoted) = &arg_expr.expr else {
            continue;
        };

        // Only flag quoted glob patterns (ones that won't expand)
        if !is_quoted {
            continue;
        }

        // Only flag if the pattern actually contains glob metacharacters
        if !has_glob_chars(pattern) {
            continue;
        }

        // Create violation
        let cmd_name = ctx.working_set.get_decl(call.decl_id).name().to_string();

        // Determine if pattern can be bare or needs glob subexpression
        let can_be_bare = !bare_glob_needs_quotes(pattern);

        let violation = Detection::from_global_span(
            format!(
                "Quoted glob pattern `\"{pattern}\"` passed to `{cmd_name}` won't expand to match \
                 files"
            ),
            arg_expr.span,
        )
        .with_primary_label("use unquoted glob or glob subexpression")
        .with_help(if can_be_bare {
            format!(
                "In Nushell, `{pattern}` without quotes expands to match files, while \
                 \"{pattern}\" with quotes is treated as a literal string pattern that won't \
                 expand. The `{cmd_name}` command typically expects glob patterns to expand and \
                 match actual files."
            )
        } else {
            "This pattern contains spaces or special characters that prevent it from being a bare \
             word."
                .to_string()
        });

        results.push((
            violation,
            FixData {
                quoted_span: arg_expr.span,
                pattern: pattern.clone(),
            },
        ));
    }

    results
}

struct GlobMayDropQuotes;

impl DetectFix for GlobMayDropQuotes {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "glob_may_drop_quotes"
    }

    fn explanation(&self) -> &'static str {
        "Glob patterns in quotes are treated as literal strings. Commands expecting glob patterns \
         work better with bare (unquoted) glob expressions for automatic file matching."
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/lang-guide/chapters/types/basic_types/glob.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            check_call(call, ctx)
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let can_be_bare = !bare_glob_needs_quotes(&fix_data.pattern);
        let replacement_text = if can_be_bare {
            fix_data.pattern.clone()
        } else {
            format!("(\"{}\" | into glob)", fix_data.pattern)
        };

        let explanation = if can_be_bare {
            format!("Remove quotes from glob pattern '{replacement_text}'")
        } else {
            format!("Convert quoted pattern to glob using '{replacement_text}'")
        };

        Some(Fix::with_explanation(
            explanation,
            vec![Replacement::new(fix_data.quoted_span, replacement_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &GlobMayDropQuotes;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
