use nu_lint_analysis::{
    LintLevel,
    ast::{
        block::BlockExt,
        call::CallExt,
        expression::{ExpressionExt, is_pipeline_input_var},
        string::format_cell_path_members,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Span,
    ast::{Call, Expr, PathMember, Pipeline},
};

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

struct FixData {
    access_span: Span,
    variable_span: Span,
    optional_members: Vec<PathMember>,
}

fn is_emptiness_check(following_call: &Call, context: &LintContext) -> bool {
    matches!(
        following_call.builtin_name(context).unwrap_or_default(),
        "is-empty" | "is-not-empty"
    )
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    detect_optional_get(pipeline, context).into_iter().collect()
}

fn detect_optional_get(pipeline: &Pipeline, context: &LintContext) -> Option<(Detection, FixData)> {
    let [variable_element, get_element, following_elements @ ..] = pipeline.elements.as_slice()
    else {
        return None;
    };

    if is_pipeline_input_var(variable_element.expr.extract_direct_var()?, context) {
        return None;
    }

    let Expr::Call(get_call) = &get_element.expr.expr else {
        return None;
    };
    if !get_call.is_get_optional(context) || get_call.positional_iter().count() != 1 {
        return None;
    }

    if let Some(Expr::Call(following_call)) = following_elements
        .first()
        .map(|following_element| &following_element.expr.expr)
        && (is_emptiness_check(following_call, context)
            || following_call.builtin_name(context) == Some("get"))
    {
        return None;
    }

    let Expr::CellPath(cell_path) = &get_call.get_first_positional_arg()?.expr else {
        return None;
    };
    if cell_path
        .members
        .iter()
        .any(|member| matches!(member, PathMember::Int { .. }))
    {
        return None;
    }

    let mut optional_members = cell_path.members.clone();
    optional_members
        .iter_mut()
        .for_each(PathMember::make_optional);

    let access_span = Span::new(variable_element.expr.span.start, get_element.expr.span.end);
    let detection =
        Detection::from_global_span("Use an optional cell path instead of `get -o`", access_span)
            .with_primary_label("replaceable with optional cell path");

    Some((
        detection,
        FixData {
            access_span,
            variable_span: variable_element.expr.span,
            optional_members,
        },
    ))
}

struct GetOptionalToCellPath;

impl DetectFix for GetOptionalToCellPath {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "get_optional_to_cell_path"
    }

    fn short_description(&self) -> &'static str {
        "Simplify `$record | get -o a.b` to `$record.a?.b?`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Reading a literal path from a variable with `get -o` needs a pipeline where an \
             optional cell path expresses the same access directly. Both forms return null for a \
             missing key, so `$record.a?.b?` is a shorter equivalent.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/navigating_structured_data.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let variable_text = context.span_text(fix_data.variable_span);
        let member_path = format_cell_path_members(&fix_data.optional_members);

        Some(Fix {
            explanation: "Read the path with an optional cell path".into(),
            replacements: vec![Replacement::new(
                fix_data.access_span,
                format!("{variable_text}.{member_path}"),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &GetOptionalToCellPath;
