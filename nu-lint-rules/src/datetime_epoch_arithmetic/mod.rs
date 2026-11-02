use nu_lint_analysis::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::{
    Type,
    ast::{Expr, Expression, Math, Operator, PipelineElement},
};

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;

const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;

fn produces_datetime(source_element: &PipelineElement, context: &LintContext) -> bool {
    source_element.expr.ty == Type::Date
        || matches!(
            &source_element.expr.expr,
            Expr::Call(source_call)
                if matches!(
                    source_call.builtin_name(context).unwrap_or_default(),
                    "date now" | "into datetime"
                )
        )
}

fn is_datetime_epoch_conversion(operand: &Expression, context: &LintContext) -> bool {
    let inner_operand = match &operand.expr {
        Expr::FullCellPath(cell_path) if cell_path.tail.is_empty() => &cell_path.head,
        _ => operand,
    };
    let Expr::Subexpression(block_id) = &inner_operand.expr else {
        return false;
    };

    let block = context.working_set.get_block(*block_id);
    let [pipeline] = block.pipelines.as_slice() else {
        return false;
    };
    let [.., source_element, conversion_element] = pipeline.elements.as_slice() else {
        return false;
    };

    matches!(
        &conversion_element.expr.expr,
        Expr::Call(conversion_call) if conversion_call.builtin_name(context) == Some("into int")
    ) && produces_datetime(source_element, context)
}

fn detect_epoch_arithmetic(
    arithmetic_expr: &Expression,
    context: &LintContext,
) -> Option<Detection> {
    let Expr::BinaryOp(left_operand, operator, right_operand) = &arithmetic_expr.expr else {
        return None;
    };
    let Expr::Operator(Operator::Math(math_operator)) = &operator.expr else {
        return None;
    };

    let is_epoch_arithmetic = match math_operator {
        Math::Subtract => {
            is_datetime_epoch_conversion(left_operand, context)
                || is_datetime_epoch_conversion(right_operand, context)
        }
        Math::Divide | Math::FloorDivide => {
            matches!(right_operand.expr, Expr::Int(NANOSECONDS_PER_SECOND))
                && is_datetime_epoch_conversion(left_operand, context)
        }
        _ => false,
    };

    is_epoch_arithmetic.then(|| {
        Detection::from_global_span(
            "Subtract datetimes directly to get a duration instead of converting them to integers",
            arithmetic_expr.span,
        )
        .with_primary_label("time arithmetic on integers")
    })
}

struct DatetimeEpochArithmetic;

impl DetectFix for DatetimeEpochArithmetic {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "datetime_epoch_arithmetic"
    }

    fn short_description(&self) -> &'static str {
        "Datetime converted to an integer for time arithmetic"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Subtracting two datetimes gives a duration that can be compared, formatted and \
             decomposed without counting nanoseconds. Converting a datetime with `into int` and \
             then subtracting or dividing by 1_000_000_000 loses that type. A plain `into int` is \
             not reported, since external tools often need an epoch value.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/types_of_data.html#dates")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect_single(detect_epoch_arithmetic))
    }
}

pub static RULE: &dyn Rule = &DatetimeEpochArithmetic;
