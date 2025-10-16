use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Block, Expr, Expression, Pipeline},
    engine::StateWorkingSet,
};

/// A visitor trait for traversing Nushell AST nodes.
pub trait AstVisitor {
    fn visit_block(&mut self, block: &Block, context: &VisitContext) {
        for pipeline in &block.pipelines {
            self.visit_pipeline(pipeline, context);
        }
    }

    fn visit_pipeline(&mut self, pipeline: &Pipeline, context: &VisitContext) {
        for element in &pipeline.elements {
            self.visit_expression(&element.expr, context);
        }
    }

    fn visit_expression(&mut self, expr: &Expression, context: &VisitContext) {
        walk_expression(self, expr, context);
    }

    fn visit_call(&mut self, call: &nu_protocol::ast::Call, context: &VisitContext) {
        walk_call(self, call, context);
    }

    fn visit_var_decl(&mut self, _var_id: VarId, _span: Span, _context: &VisitContext) {}

    fn visit_var_ref(&mut self, _var_id: VarId, _span: Span, _context: &VisitContext) {}

    fn visit_binary_op(
        &mut self,
        lhs: &Expression,
        op: &Expression,
        rhs: &Expression,
        context: &VisitContext,
    ) {
        self.visit_expression(lhs, context);
        self.visit_expression(op, context);
        self.visit_expression(rhs, context);
    }

    fn visit_list(&mut self, items: &[nu_protocol::ast::ListItem], context: &VisitContext) {
        for item in items {
            match item {
                nu_protocol::ast::ListItem::Item(expr)
                | nu_protocol::ast::ListItem::Spread(_, expr) => {
                    self.visit_expression(expr, context);
                }
            }
        }
    }

    fn visit_string(&mut self, _content: &str, _span: Span, _context: &VisitContext) {}

    fn visit_int(&mut self, _value: i64, _span: Span, _context: &VisitContext) {}
}

/// Context information available during AST traversal
pub struct VisitContext<'a> {
    pub working_set: &'a StateWorkingSet<'a>,
    pub source: &'a str,
}

impl<'a> VisitContext<'a> {
    #[must_use]
    pub fn new(working_set: &'a StateWorkingSet<'a>, source: &'a str) -> Self {
        Self {
            working_set,
            source,
        }
    }

    /// Get the text content of a span
    #[must_use]
    pub fn get_span_contents(&self, span: Span) -> &str {
        crate::parser::get_span_contents(self.source, span)
    }

    /// Get a block by its ID
    #[must_use]
    pub fn get_block(&self, block_id: BlockId) -> &Block {
        self.working_set.get_block(block_id)
    }

    /// Get variable info by ID
    #[must_use]
    pub fn get_variable(&self, var_id: VarId) -> &nu_protocol::engine::Variable {
        self.working_set.get_variable(var_id)
    }

    /// Extract text arguments from an external command call
    /// Returns a Vec of argument strings extracted from the AST
    #[must_use]
    pub fn extract_external_args(
        &self,
        args: &[nu_protocol::ast::ExternalArgument],
    ) -> Vec<String> {
        args.iter()
            .map(|arg| match &arg {
                nu_protocol::ast::ExternalArgument::Regular(expr) => {
                    self.get_span_contents(expr.span).to_string()
                }
                nu_protocol::ast::ExternalArgument::Spread(expr) => {
                    format!("...{}", self.get_span_contents(expr.span))
                }
            })
            .collect()
    }
}

/// Continue walking expression children after custom logic
pub fn walk_expression<V: AstVisitor + ?Sized>(
    visitor: &mut V,
    expr: &Expression,
    context: &VisitContext,
) {
    match &expr.expr {
        Expr::VarDecl(var_id) => visitor.visit_var_decl(*var_id, expr.span, context),
        Expr::Var(var_id) => visitor.visit_var_ref(*var_id, expr.span, context),
        Expr::Call(call) => visitor.visit_call(call, context),
        Expr::BinaryOp(lhs, op, rhs) => visitor.visit_binary_op(lhs, op, rhs, context),
        Expr::UnaryNot(inner) => visitor.visit_expression(inner, context),
        Expr::List(items) => visitor.visit_list(items, context),
        Expr::Record(items) => {
            for item in items {
                match item {
                    nu_protocol::ast::RecordItem::Pair(key, value) => {
                        visitor.visit_expression(key, context);
                        visitor.visit_expression(value, context);
                    }
                    nu_protocol::ast::RecordItem::Spread(_, expr) => {
                        visitor.visit_expression(expr, context);
                    }
                }
            }
        }
        Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
            let block = context.get_block(*block_id);
            visitor.visit_block(block, context);
        }
        Expr::FullCellPath(cell_path) => visitor.visit_expression(&cell_path.head, context),
        Expr::String(string_content) | Expr::RawString(string_content) => {
            visitor.visit_string(string_content, expr.span, context);
        }
        Expr::Int(value) => visitor.visit_int(*value, expr.span, context),
        Expr::StringInterpolation(exprs) => {
            for expr in exprs {
                visitor.visit_expression(expr, context);
            }
        }
        Expr::MatchBlock(arms) => {
            for (_, expr) in arms {
                visitor.visit_expression(expr, context);
            }
        }
        _ => {}
    }
}

/// Continue walking call arguments after custom logic
pub fn walk_call<V: AstVisitor + ?Sized>(
    visitor: &mut V,
    call: &nu_protocol::ast::Call,
    context: &VisitContext,
) {
    for arg in &call.arguments {
        match arg {
            nu_protocol::ast::Argument::Named(named) => {
                if let Some(expr) = &named.2 {
                    visitor.visit_expression(expr, context);
                }
            }
            nu_protocol::ast::Argument::Positional(expr)
            | nu_protocol::ast::Argument::Unknown(expr)
            | nu_protocol::ast::Argument::Spread(expr) => {
                visitor.visit_expression(expr, context);
            }
        }
    }
}
