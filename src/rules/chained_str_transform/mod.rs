use nu_protocol::{
    Span,
    ast::{Argument, Call, Expr, Pipeline},
};

use crate::{
    LintLevel,
    ast::{
        block::BlockExt,
        call::CallExt,
        pipeline::{ClusterConfig, PipelineExt},
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

/// Fix data for combining str replace calls
struct FixData {
    span: Span,
    patterns: Vec<String>,
    replacement: String,
}

fn extract_args(call: &Call) -> Option<(String, String)> {
    let pos: Vec<_> = call
        .arguments
        .iter()
        .filter_map(|a| match a {
            Argument::Positional(e) | Argument::Unknown(e) => Some(e),
            _ => None,
        })
        .collect();

    let find = match &pos.first()?.expr {
        Expr::String(s) | Expr::RawString(s) => s.clone(),
        _ => return None,
    };
    let repl = match &pos.get(1)?.expr {
        Expr::String(s) | Expr::RawString(s) => s.clone(),
        _ => return None,
    };
    Some((find, repl))
}

fn escape_regex(s: &str) -> String {
    s.chars().fold(String::new(), |mut r, c| {
        if "\\^$.|?*+()[]{}".contains(c) {
            r.push('\\');
        }
        r.push(c);
        r
    })
}

fn patterns_overlap(patterns: &[String]) -> bool {
    patterns.iter().enumerate().any(|(i, p1)| {
        patterns
            .iter()
            .enumerate()
            .any(|(j, p2)| i != j && p1.contains(p2.as_str()))
    })
}

fn try_build_fix(calls: &[&Call], span: Span) -> Option<FixData> {
    let mut finds = Vec::new();
    let mut replacement = None;

    for call in calls {
        let (find, repl) = extract_args(call)?;
        if !call.has_named_flag("all") && !call.has_named_flag("a") {
            return None;
        }
        if call.has_named_flag("regex") || call.has_named_flag("r") {
            return None;
        }
        if replacement.get_or_insert(repl.clone()) != &repl {
            return None;
        }
        finds.push(find);
    }

    if patterns_overlap(&finds) {
        return None;
    }

    Some(FixData {
        span,
        patterns: finds.iter().map(|s| escape_regex(s)).collect(),
        replacement: replacement?,
    })
}

fn find_clusters_in_pipeline(
    pipeline: &Pipeline,
    ctx: &LintContext,
) -> Vec<(Detection, Option<FixData>)> {
    let config = ClusterConfig::min_consecutive(2);

    pipeline
        .find_command_clusters("str replace", ctx, &config)
        .into_iter()
        .map(|cluster| {
            let fix = try_build_fix(&cluster.calls, cluster.span);

            let msg = format!(
                "{} consecutive 'str replace' calls. {}",
                cluster.len(),
                if fix.is_some() {
                    "Can be combined with regex alternation"
                } else {
                    "Consider combining if patterns share the same replacement"
                }
            );
            (Detection::from_global_span(msg, cluster.span), fix)
        })
        .collect()
}

struct ChainedStrReplace;

impl DetectFix for ChainedStrReplace {
    type FixInput<'a> = Option<FixData>;

    fn id(&self) -> &'static str {
        "chained_str_transform"
    }

    fn short_description(&self) -> &'static str {
        "Consecutive `str replace` combinable"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Multiple consecutive 'str replace -a' calls with the same replacement value can be \
             combined into a single 'str replace -ar' using regex alternation (e.g., 'a|b'). This \
             processes the string once and produces cleaner code. Auto-fix applies when all calls \
             use '-a', have the same replacement, use literal patterns, and patterns don't \
             overlap (no pattern is a substring of another).",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/str_replace.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .ast
            .detect_in_pipelines(context, find_clusters_in_pipeline)
    }

    fn fix(&self, _context: &LintContext, data: &Self::FixInput<'_>) -> Option<Fix> {
        let d = data.as_ref()?;
        let pattern = d.patterns.join("|");
        let replacement = format!("str replace -ar \"{}\" \"{}\"", pattern, d.replacement);
        Some(Fix {
            explanation: "combine".into(),
            replacements: vec![Replacement::new(d.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &ChainedStrReplace;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
