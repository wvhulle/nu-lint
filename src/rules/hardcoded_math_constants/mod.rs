use std::{f64::consts, ops::ControlFlow};

use nu_protocol::{
    Span,
    ast::{Expr, Expression},
};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const MIN_DIGITS: usize = 5;
const MIN_MATCHING_DIGITS: usize = 3;
const RELATIVE_ERROR_THRESHOLD: f64 = 0.0001;

struct MathConstant {
    name: &'static str,
    value: f64,
}

const MATH_CONSTANTS: &[MathConstant] = &[
    MathConstant {
        name: "PI",
        value: consts::PI,
    },
    MathConstant {
        name: "E",
        value: consts::E,
    },
    MathConstant {
        name: "TAU",
        value: consts::TAU,
    },
    MathConstant {
        name: "PHI",
        value: 1.618_033_988_749_895,
    },
    MathConstant {
        name: "GAMMA",
        value: 0.577_215_664_901_532_9,
    },
];

fn matches_constant(value: f64) -> Option<&'static str> {
    let abs_value = value.abs();

    let value_str = abs_value.to_string();
    let value_digits: String = value_str.chars().filter(char::is_ascii_digit).collect();

    if value_digits.len() < MIN_DIGITS {
        return None;
    }

    for constant in MATH_CONSTANTS {
        let diff = (abs_value - constant.value).abs();
        let relative_error = diff / constant.value;

        if relative_error < RELATIVE_ERROR_THRESHOLD {
            let const_str = constant.value.to_string();
            let const_digits: String = const_str.chars().filter(char::is_ascii_digit).collect();

            let matching_digits = value_digits
                .chars()
                .zip(const_digits.chars())
                .take_while(|(a, b)| a == b)
                .count();

            if matching_digits >= MIN_MATCHING_DIGITS {
                return Some(constant.name);
            }
        }
    }
    None
}

struct FixData {
    constant_name: &'static str,
    span: Span,
}

struct HardcodedMathConstants;

impl DetectFix for HardcodedMathConstants {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "hardcoded_math_constants"
    }

    fn short_description(&self) -> &'static str {
        "Hardcoded mathematical constants should use std/math constants instead"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/modules.html#using-modules")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut results = Vec::new();

        context.traverse_with_parent(|expr: &Expression, _parent| {
            if let Expr::Float(value) = expr.expr
                && let Some(constant_name) = matches_constant(value)
            {
                let detection = Detection::from_global_span(
                    format!(
                        "Hardcoded mathematical constant detected. Use $math.{constant_name} from \
                         std/math instead of {value}"
                    ),
                    expr.span,
                )
                .with_primary_label("hardcoded constant");

                let fix_data = FixData {
                    constant_name,
                    span: expr.span,
                };
                results.push((detection, fix_data));
            }
            ControlFlow::Continue(())
        });

        results
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Replace with $math.{}", fix_data.constant_name).into(),
            replacements: vec![Replacement::new(
                fix_data.span,
                format!("$math.{}", fix_data.constant_name),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &HardcodedMathConstants;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
