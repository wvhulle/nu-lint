use nu_lint_analysis::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    DeclId, Span,
    ast::{Argument, Call, Expr, Expression},
};

struct ShortFlagInfo {
    short_char: char,
    decl_id: DeclId,
    flag_span: Span,
}

const SINGLE_SHORT_FLAG_LENGTH: usize = 2;

const fn is_single_short_flag(flag_span: Span) -> bool {
    flag_span.end - flag_span.start == SINGLE_SHORT_FLAG_LENGTH
}

fn long_name_of_short_flag(
    decl_id: DeclId,
    short_char: char,
    context: &LintContext,
) -> Option<String> {
    context
        .working_set
        .get_decl(decl_id)
        .signature()
        .named
        .into_iter()
        .find(|flag| flag.short == Some(short_char))
        .map(|flag| flag.long)
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
            let long_name = &spanned_name.item;

            if is_single_short_flag(flag_span) {
                // The parser has already resolved it to the long name
                // We need to find what short char was used
                if let Some(short_char) = short_alias.as_ref().and_then(|s| s.item.chars().next()) {
                    short_flags.push(ShortFlagInfo {
                        short_char,
                        decl_id: call.decl_id,
                        flag_span,
                    });
                } else if let Some(short_char) = find_short_for_long(call, long_name, context) {
                    // Fall back to looking up from signature
                    short_flags.push(ShortFlagInfo {
                        short_char,
                        decl_id: call.decl_id,
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
        .filter_map(|info| {
            let long_name = long_name_of_short_flag(info.decl_id, info.short_char, context)?;
            let detection = Detection::from_global_span(
                format!(
                    "Short flag '-{}' used in '{cmd_name}', consider using '--{long_name}' for \
                     clarity",
                    info.short_char
                ),
                info.flag_span,
            )
            .with_primary_label(format!("use '--{long_name}' instead"));

            Some((detection, info))
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
            "Short flags like `-a` or `-f` are convenient for interactive use but reduce code \
             readability in scripts. Long flags like `--all` or `--force` are self-documenting \
             and make the code easier to understand for others reading it. ",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        None
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| check_call(expr, ctx))
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let long_name = long_name_of_short_flag(fix_data.decl_id, fix_data.short_char, context)?;
        let replacement_text = format!("--{long_name}");
        Some(Fix {
            explanation: format!("Replace '-{}' with '--{long_name}'", fix_data.short_char).into(),
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
