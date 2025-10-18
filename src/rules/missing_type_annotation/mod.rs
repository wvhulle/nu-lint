use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
    visitor::{AstVisitor, VisitContext},
};

struct TypeAnnotationVisitor {
    violations: Vec<Violation>,
}

impl TypeAnnotationVisitor {
    fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn check_signature(&mut self, sig: &nu_protocol::Signature, _context: &VisitContext) {
        // Check required positional parameters
        for param in &sig.required_positional {
            if param.shape == nu_protocol::SyntaxShape::Any {
                self.violations.push(Violation {
                    rule_id: "missing_type_annotation".into(),
                    severity: Severity::Info,
                    message: format!("Parameter '{}' is missing type annotation", param.name).into(),
                    span: nu_protocol::Span::unknown(), /* Signature doesn't store individual
                                                         * param spans */
                    suggestion: Some(
                        "Add type annotation like 'param: string' or 'param: int'".to_string().into(),
                    ),
                    fix: None,
                    file: None,
                });
            }
        }

        // Check optional positional parameters
        for param in &sig.optional_positional {
            if param.shape == nu_protocol::SyntaxShape::Any {
                self.violations.push(Violation {
                    rule_id: "missing_type_annotation".into(),
                    severity: Severity::Info,
                    message: format!("Parameter '{}' is missing type annotation", param.name).into(),
                    span: nu_protocol::Span::unknown(),
                    suggestion: Some(
                        "Add type annotation like 'param: string' or 'param: int'".to_string().into(),
                    ),
                    fix: None,
                    file: None,
                });
            }
        }

        // Check rest positional parameter
        if let Some(param) = &sig.rest_positional
            && param.shape == nu_protocol::SyntaxShape::Any
        {
            self.violations.push(Violation {
                rule_id: "missing_type_annotation".into(),
                severity: Severity::Info,
                message: format!("Parameter '{}' is missing type annotation", param.name).into(),
                span: nu_protocol::Span::unknown(),
                suggestion: Some(
                    "Add type annotation like 'param: string' or 'param: list'".to_string().into(),
                ),
                fix: None,
                file: None,
            });
        }
    }
}

impl AstVisitor for TypeAnnotationVisitor {
    fn visit_call(&mut self, call: &nu_protocol::ast::Call, context: &VisitContext) {
        // Check if this is a def command
        let decl = context.get_decl(call.decl_id);
        if decl.name() == "def" {
            // The second positional argument of 'def' is a Signature expression
            // def name [params] { body }
            for arg in &call.arguments {
                if let nu_protocol::ast::Argument::Positional(arg_expr) = arg
                    && let Expr::Signature(sig) = &arg_expr.expr
                {
                    self.check_signature(sig, context);
                }
            }
        }

        crate::visitor::walk_call(self, call, context);
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut visitor = TypeAnnotationVisitor::new();
    context.walk_ast(&mut visitor);
    visitor.into_violations()
}

pub fn rule() -> Rule {
    Rule::new(
        "missing_type_annotation",
        RuleCategory::TypeSafety,
        Severity::Info,
        "Parameters should have type annotations",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
