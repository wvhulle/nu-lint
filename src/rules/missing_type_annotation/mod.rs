use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
    visitor::{AstVisitor, VisitContext},
};

pub struct MissingTypeAnnotation;

impl MissingTypeAnnotation {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for MissingTypeAnnotation {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MissingTypeAnnotation {
    fn id(&self) -> &'static str {
        "missing_type_annotation"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::TypeSafety
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Parameters should have type annotations"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = TypeAnnotationVisitor::new(self.id().to_string(), self.severity());
        context.walk_ast(&mut visitor);
        visitor.into_violations()
    }
}

struct TypeAnnotationVisitor {
    rule_id: String,
    severity: Severity,
    violations: Vec<Violation>,
}

impl TypeAnnotationVisitor {
    fn new(rule_id: String, severity: Severity) -> Self {
        Self {
            rule_id,
            severity,
            violations: Vec::new(),
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
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

impl TypeAnnotationVisitor {
    fn check_signature(&mut self, sig: &nu_protocol::Signature, _context: &VisitContext) {
        // Check required positional parameters
        for param in &sig.required_positional {
            if param.shape == nu_protocol::SyntaxShape::Any {
                self.violations.push(Violation {
                    rule_id: self.rule_id.clone(),
                    severity: self.severity,
                    message: format!("Parameter '{}' is missing type annotation", param.name),
                    span: nu_protocol::Span::unknown(), // Signature doesn't store individual param spans
                    suggestion: Some(
                        "Add type annotation like 'param: string' or 'param: int'".to_string(),
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
                    rule_id: self.rule_id.clone(),
                    severity: self.severity,
                    message: format!("Parameter '{}' is missing type annotation", param.name),
                    span: nu_protocol::Span::unknown(),
                    suggestion: Some(
                        "Add type annotation like 'param: string' or 'param: int'".to_string(),
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
                rule_id: self.rule_id.clone(),
                severity: self.severity,
                message: format!("Parameter '{}' is missing type annotation", param.name),
                span: nu_protocol::Span::unknown(),
                suggestion: Some(
                    "Add type annotation like 'param: string' or 'param: list'".to_string(),
                ),
                fix: None,
                file: None,
            });
        }
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
