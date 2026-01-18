use nu_protocol::{
    Span,
    ast::{Argument, Call, Expr, Expression},
};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct ShortFlagInfo {
    short_char: char,
    long_name: String,
    flag_span: Span,
}

/// Checks if the span represents a short flag (starts with single dash and is
/// short)
fn is_short_flag_span(context: &LintContext, span: Span) -> bool {
    let text = context.span_text(span);
    // Short flags are like "-a", "-f", etc. (2-3 chars with single dash)
    // Long flags are like "--all", "--force", etc.
    text.starts_with('-') && !text.starts_with("--") && text.len() <= 3
}

/// Finds the short character for a given long flag name from the command
/// signature
fn find_short_for_long(call: &Call, long_name: &str, context: &LintContext) -> Option<char> {
    let decl = context.working_set.get_decl(call.decl_id);
    let signature = decl.signature();

    signature
        .named
        .iter()
        .find(|flag| flag.long == long_name)
        .and_then(|flag| flag.short)
}

fn extract_short_flags(call: &Call, context: &LintContext) -> Vec<ShortFlagInfo> {
    let mut short_flags = Vec::new();

    for arg in &call.arguments {
        if let Argument::Named((spanned_name, short_alias, _value)) = arg {
            let flag_span = spanned_name.span;
            let long_name = spanned_name.item.clone();

            // Check if this was written as a short flag by examining the source text
            if is_short_flag_span(context, flag_span) {
                // The parser has already resolved it to the long name
                // We need to find what short char was used
                if let Some(short_char) = short_alias.as_ref().and_then(|s| s.item.chars().next()) {
                    short_flags.push(ShortFlagInfo {
                        short_char,
                        long_name,
                        flag_span,
                    });
                } else if let Some(short_char) = find_short_for_long(call, &long_name, context) {
                    // Fall back to looking up from signature
                    short_flags.push(ShortFlagInfo {
                        short_char,
                        long_name,
                        flag_span,
                    });
                }
            }
        }
    }

    short_flags
}

fn check_call(expr: &Expression, context: &LintContext) -> Vec<(Detection, ShortFlagInfo)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    // Only check builtin commands, not custom commands
    let decl = context.working_set.get_decl(call.decl_id);
    if !decl.is_builtin() {
        return vec![];
    }

    let cmd_name = call.get_call_name(context);
    let short_flags = extract_short_flags(call, context);

    short_flags
        .into_iter()
        .map(|info| {
            let detection = Detection::from_global_span(
                format!(
                    "Short flag '-{}' used in '{}', consider using '--{}' for clarity",
                    info.short_char, cmd_name, info.long_name
                ),
                info.flag_span,
            )
            .with_primary_label(format!("use '--{}' instead", info.long_name));

            (detection, info)
        })
        .collect()
}

struct PreferLongFlags;

impl DetectFix for PreferLongFlags {
    type FixInput<'a> = ShortFlagInfo;

    fn id(&self) -> &'static str {
        "explicit_long_flags"
    }

    fn short_description(&self) -> &'static str {
        "Replace short flags (-f) with long flags (--flag)"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Short flags like '-a' or '-f' are convenient for interactive use but reduce code \
             readability in scripts. Long flags like '--all' or '--force' are self-documenting \
             and make the code easier to understand for others reading it. ",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        None
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| check_call(expr, ctx))
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement_text = format!("--{}", fix_data.long_name);
        Some(Fix {
            explanation: format!(
                "Replace '-{}' with '--{}'",
                fix_data.short_char, fix_data.long_name
            )
            .into(),
            replacements: vec![Replacement::new(fix_data.flag_span, replacement_text)],
        })
    }
}

pub static RULE: &dyn Rule = &PreferLongFlags;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
