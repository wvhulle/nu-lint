//! Combined AST visitor for single-pass traversal optimization
//!
//! This module provides infrastructure to traverse the AST once while running
//! multiple rules, reducing overhead from repeated tree walking.
//!
//! # Status
//! Infrastructure is complete but not currently activated in the engine.
//! See `COMBINED_TRAVERSAL.md` for details.

use nu_protocol::{
    Span, VarId,
    ast::{Block, Call, Expression, Pipeline},
};

use crate::{
    lint::Violation,
    rules::{
        missing_type_annotation::TypeAnnotationVisitor, pipe_spacing::PipeSpacingVisitor,
        prefer_compound_assignment::CompoundAssignmentVisitor,
        prefer_parse_over_each_split::EachSplitVisitor, unnecessary_mut::MutVariableVisitor,
    },
    visitor::{AstVisitor, VisitContext},
};

/// A combined visitor that holds specific visitor types and collects their
/// violations
pub struct CombinedAstVisitor<'a> {
    pipe_spacing: Option<PipeSpacingVisitor<'a>>,
    type_annotation: Option<TypeAnnotationVisitor>,
    unnecessary_mut: Option<MutVariableVisitor<'a>>,
    each_split: Option<EachSplitVisitor<'a>>,
    compound_assignment: Option<CompoundAssignmentVisitor<'a>>,
}

impl<'a> CombinedAstVisitor<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipe_spacing: None,
            type_annotation: None,
            unnecessary_mut: None,
            each_split: None,
            compound_assignment: None,
        }
    }

    pub fn set_pipe_spacing(&mut self, visitor: PipeSpacingVisitor<'a>) {
        self.pipe_spacing = Some(visitor);
    }

    pub fn set_type_annotation(&mut self, visitor: TypeAnnotationVisitor) {
        self.type_annotation = Some(visitor);
    }

    pub fn set_unnecessary_mut(&mut self, visitor: MutVariableVisitor<'a>) {
        self.unnecessary_mut = Some(visitor);
    }

    pub fn set_each_split(&mut self, visitor: EachSplitVisitor<'a>) {
        self.each_split = Some(visitor);
    }

    pub fn set_compound_assignment(&mut self, visitor: CompoundAssignmentVisitor<'a>) {
        self.compound_assignment = Some(visitor);
    }

    /// Collect all violations from the visitors
    #[must_use]
    pub fn collect_violations(self) -> Vec<Violation> {
        let mut violations = Vec::new();

        if let Some(mut v) = self.pipe_spacing {
            violations.extend(v.take_violations());
        }
        if let Some(v) = self.type_annotation {
            violations.extend(v.into_violations());
        }
        if let Some(mut v) = self.unnecessary_mut {
            violations.extend(v.take_violations());
        }
        if let Some(mut v) = self.each_split {
            violations.extend(v.take_violations());
        }
        if let Some(mut v) = self.compound_assignment {
            violations.extend(v.take_violations());
        }

        violations
    }
}

impl Default for CombinedAstVisitor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl AstVisitor for CombinedAstVisitor<'_> {
    fn visit_block(&mut self, block: &Block, context: &VisitContext) {
        if let Some(v) = &mut self.pipe_spacing {
            v.visit_block(block, context);
        }
        if let Some(v) = &mut self.type_annotation {
            v.visit_block(block, context);
        }
        if let Some(v) = &mut self.unnecessary_mut {
            v.visit_block(block, context);
        }
        if let Some(v) = &mut self.each_split {
            v.visit_block(block, context);
        }
        if let Some(v) = &mut self.compound_assignment {
            v.visit_block(block, context);
        }
    }

    fn visit_pipeline(&mut self, pipeline: &Pipeline, context: &VisitContext) {
        if let Some(v) = &mut self.pipe_spacing {
            v.visit_pipeline(pipeline, context);
        }
        if let Some(v) = &mut self.type_annotation {
            v.visit_pipeline(pipeline, context);
        }
        if let Some(v) = &mut self.unnecessary_mut {
            v.visit_pipeline(pipeline, context);
        }
        if let Some(v) = &mut self.each_split {
            v.visit_pipeline(pipeline, context);
        }
        if let Some(v) = &mut self.compound_assignment {
            v.visit_pipeline(pipeline, context);
        }
    }

    fn visit_expression(&mut self, expr: &Expression, context: &VisitContext) {
        if let Some(v) = &mut self.pipe_spacing {
            v.visit_expression(expr, context);
        }
        if let Some(v) = &mut self.type_annotation {
            v.visit_expression(expr, context);
        }
        if let Some(v) = &mut self.unnecessary_mut {
            v.visit_expression(expr, context);
        }
        if let Some(v) = &mut self.each_split {
            v.visit_expression(expr, context);
        }
        if let Some(v) = &mut self.compound_assignment {
            v.visit_expression(expr, context);
        }
    }

    fn visit_call(&mut self, call: &Call, context: &VisitContext) {
        if let Some(v) = &mut self.pipe_spacing {
            v.visit_call(call, context);
        }
        if let Some(v) = &mut self.type_annotation {
            v.visit_call(call, context);
        }
        if let Some(v) = &mut self.unnecessary_mut {
            v.visit_call(call, context);
        }
        if let Some(v) = &mut self.each_split {
            v.visit_call(call, context);
        }
        if let Some(v) = &mut self.compound_assignment {
            v.visit_call(call, context);
        }
    }

    fn visit_var_decl(&mut self, var_id: VarId, span: Span, context: &VisitContext) {
        if let Some(v) = &mut self.pipe_spacing {
            v.visit_var_decl(var_id, span, context);
        }
        if let Some(v) = &mut self.type_annotation {
            v.visit_var_decl(var_id, span, context);
        }
        if let Some(v) = &mut self.unnecessary_mut {
            v.visit_var_decl(var_id, span, context);
        }
        if let Some(v) = &mut self.each_split {
            v.visit_var_decl(var_id, span, context);
        }
        if let Some(v) = &mut self.compound_assignment {
            v.visit_var_decl(var_id, span, context);
        }
    }

    fn visit_var_ref(&mut self, var_id: VarId, span: Span, context: &VisitContext) {
        if let Some(v) = &mut self.pipe_spacing {
            v.visit_var_ref(var_id, span, context);
        }
        if let Some(v) = &mut self.type_annotation {
            v.visit_var_ref(var_id, span, context);
        }
        if let Some(v) = &mut self.unnecessary_mut {
            v.visit_var_ref(var_id, span, context);
        }
        if let Some(v) = &mut self.each_split {
            v.visit_var_ref(var_id, span, context);
        }
        if let Some(v) = &mut self.compound_assignment {
            v.visit_var_ref(var_id, span, context);
        }
    }

    fn visit_binary_op(
        &mut self,
        lhs: &Expression,
        op: &Expression,
        rhs: &Expression,
        context: &VisitContext,
    ) {
        if let Some(v) = &mut self.pipe_spacing {
            v.visit_binary_op(lhs, op, rhs, context);
        }
        if let Some(v) = &mut self.type_annotation {
            v.visit_binary_op(lhs, op, rhs, context);
        }
        if let Some(v) = &mut self.unnecessary_mut {
            v.visit_binary_op(lhs, op, rhs, context);
        }
        if let Some(v) = &mut self.each_split {
            v.visit_binary_op(lhs, op, rhs, context);
        }
        if let Some(v) = &mut self.compound_assignment {
            v.visit_binary_op(lhs, op, rhs, context);
        }
    }

    fn visit_list(&mut self, items: &[nu_protocol::ast::ListItem], context: &VisitContext) {
        if let Some(v) = &mut self.pipe_spacing {
            v.visit_list(items, context);
        }
        if let Some(v) = &mut self.type_annotation {
            v.visit_list(items, context);
        }
        if let Some(v) = &mut self.unnecessary_mut {
            v.visit_list(items, context);
        }
        if let Some(v) = &mut self.each_split {
            v.visit_list(items, context);
        }
        if let Some(v) = &mut self.compound_assignment {
            v.visit_list(items, context);
        }
    }
}
