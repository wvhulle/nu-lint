use std::iter::repeat_with;

use nu_protocol::{
    Span, SyntaxShape,
    ast::{Argument, Call, Expr, Expression, ListItem, RecordItem},
};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    span: Span,
}

fn is_bare_nothing(expr: &Expression) -> bool {
    match &expr.expr {
        Expr::String(content) => content == "nothing" && expr.span.len() == content.len(),
        Expr::GlobPattern(content, false) => content == "nothing",
        _ => false,
    }
}

fn untyped_positional_args<'a>(call: &'a Call, context: &LintContext) -> Vec<&'a Expression> {
    let signature = context.working_set.get_decl(call.decl_id).signature();
    let fixed_shapes = signature
        .required_positional
        .iter()
        .chain(&signature.optional_positional)
        .map(|param| param.shape.clone());
    let rest_shape = signature.rest_positional.map(|param| param.shape);
    let shapes = fixed_shapes.chain(repeat_with(move || rest_shape.clone()).flatten());
    call.arguments
        .iter()
        .filter_map(|arg| match arg {
            Argument::Positional(value) => Some(value),
            _ => None,
        })
        .zip(shapes)
        .filter(|(_, shape)| *shape == SyntaxShape::Any)
        .map(|(value, _)| value)
        .collect()
}

fn value_positions<'a>(expr: &'a Expression, context: &LintContext) -> Vec<&'a Expression> {
    match &expr.expr {
        Expr::Call(call) => untyped_positional_args(call, context),
        Expr::Record(items) => items
            .iter()
            .filter_map(|item| match item {
                RecordItem::Pair(_, value) => Some(value),
                RecordItem::Spread(..) => None,
            })
            .collect(),
        Expr::List(items) => items
            .iter()
            .filter_map(|item| match item {
                ListItem::Item(value) => Some(value),
                ListItem::Spread(..) => None,
            })
            .collect(),
        Expr::Table(table) => table.rows.iter().flat_map(|row| row.iter()).collect(),
        Expr::BinaryOp(lhs, _, rhs) => vec![lhs, rhs],
        Expr::ExternalCall(head, args) if args.is_empty() => vec![head],
        _ => vec![],
    }
}

struct NothingOutsideFunctionSignature;

impl DetectFix for NothingOutsideFunctionSignature {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "nothing_outside_signature"
    }

    fn short_description(&self) -> &'static str {
        "`nothing` type used outside signature"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/types.html#nothing")
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            r#"'nothing' is a type annotation keyword for function signatures. It indicates that a function returns no value.

To represent the absence of a value in a function body, use 'null' instead."#,
        )
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, context| {
            value_positions(expr, context)
                .into_iter()
                .filter(|value| is_bare_nothing(value))
                .map(|value| {
                    let detection = Detection::from_global_span(
                        "The keyword 'nothing' should only be used in function signatures, not in \
                         expressions or function bodies",
                        value.span,
                    )
                    .with_primary_label("'nothing' used outside function signature");
                    (detection, FixData { span: value.span })
                })
                .collect()
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: "Replace 'nothing' with 'null'".into(),
            replacements: vec![Replacement::new(fix_data.span, "null".to_string())],
        })
    }
}

pub static RULE: &dyn Rule = &NothingOutsideFunctionSignature;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
