use nu_protocol::{
    Span,
    ast::{Argument, Expr, Expression, Pipeline},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt},
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

fn is_str_replace(expr: &Expression, ctx: &LintContext) -> bool {
    matches!(&expr.expr, Expr::Call(c) if c.is_call_to_command("str replace", ctx))
}

fn has_flag(expr: &Expression, flag: &str) -> bool {
    let Expr::Call(c) = &expr.expr else {
        return false;
    };
    c.has_named_flag(flag)
}

fn extract_args(expr: &Expression) -> Option<(String, String)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };
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

fn try_build_fix(cluster: &[&Expression]) -> Option<FixData> {
    let mut finds = Vec::new();
    let mut replacement = None;

    for expr in cluster {
        let (find, repl) = extract_args(expr)?;
        if !has_flag(expr, "all") && !has_flag(expr, "a") {
            return None;
        }
        if has_flag(expr, "regex") || has_flag(expr, "r") {
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
        span: Span::new(cluster.first()?.span.start, cluster.last()?.span.end),
        patterns: finds.iter().map(|s| escape_regex(s)).collect(),
        replacement: replacement?,
    })
}

fn find_clusters_in_pipeline(
    pipeline: &Pipeline,
    ctx: &LintContext,
) -> Vec<(Detection, Option<FixData>)> {
    let elements = &pipeline.elements;
    let is_replace = |i: usize| is_str_replace(&elements[i].expr, ctx);

    // Find consecutive runs of str replace calls
    let mut clusters = Vec::new();
    let mut start = 0;

    while start < elements.len() {
        if !is_replace(start) {
            start += 1;
            continue;
        }

        let end = (start..elements.len())
            .take_while(|&i| is_replace(i))
            .last()
            .unwrap_or(start)
            + 1;

        if end - start >= 2 {
            clusters.push(start..end);
        }
        start = end;
    }

    clusters
        .into_iter()
        .map(|range| {
            let cluster: Vec<_> = elements[range].iter().map(|e| &e.expr).collect();
            let span = Span::new(
                cluster.first().unwrap().span.start,
                cluster.last().unwrap().span.end,
            );
            let fix = try_build_fix(&cluster);

            let msg = format!(
                "{} consecutive 'str replace' calls. {}",
                cluster.len(),
                if fix.is_some() {
                    "Can be combined with regex alternation"
                } else {
                    "Consider combining if patterns share the same replacement"
                }
            );
            (Detection::from_global_span(msg, span), fix)
        })
        .collect()
}

struct ChainedStrReplace;

impl DetectFix for ChainedStrReplace {
    type FixInput<'a> = Option<FixData>;

    fn id(&self) -> &'static str {
        "chained_str_replace"
    }

    fn short_description(&self) -> &'static str {
        "Multiple sequential 'str replace' calls can often be combined"
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

    fn level(&self) -> LintLevel {
        LintLevel::Hint
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
        Some(Fix::with_explanation(
            format!("Combine into: {replacement}"),
            vec![Replacement::new(d.span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &ChainedStrReplace;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
