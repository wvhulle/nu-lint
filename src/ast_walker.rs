use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Block, Expr, Expression, Pipeline},
    engine::StateWorkingSet,
};

/// A visitor trait for traversing Nushell AST nodes.
/// Implement this trait to define custom logic for each type of AST node.
pub trait AstVisitor {
    /// Called when visiting an expression
    fn visit_expression(&mut self, expr: &Expression, context: &VisitContext) {
        walk_expression(self, expr, context);
    }

    /// Called when visiting a pipeline
    fn visit_pipeline(&mut self, pipeline: &Pipeline, context: &VisitContext) {
        walk_pipeline(self, pipeline, context);
    }

    /// Called when visiting a block
    fn visit_block(&mut self, block: &Block, context: &VisitContext) {
        walk_block(self, block, context);
    }

    /// Called when visiting a variable declaration
    fn visit_var_decl(&mut self, var_id: VarId, span: Span, context: &VisitContext) {
        let _ = (var_id, span, context);
    }

    /// Called when visiting a function call
    fn visit_call(&mut self, call: &nu_protocol::ast::Call, context: &VisitContext) {
        walk_call(self, call, context);
    }

    /// Called when visiting a variable reference
    fn visit_var_ref(&mut self, var_id: VarId, span: Span, context: &VisitContext) {
        let _ = (var_id, span, context);
    }

    /// Called when visiting a binary operation
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

    /// Called when visiting a list
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

    /// Called when visiting a string literal
    fn visit_string(&mut self, content: &str, span: Span, context: &VisitContext) {
        let _ = (content, span, context);
    }

    /// Called when visiting an integer literal
    fn visit_int(&mut self, value: i64, span: Span, context: &VisitContext) {
        let _ = (value, span, context);
    }
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

/// Walk through a block, visiting all pipelines
pub fn walk_block<V: AstVisitor + ?Sized>(visitor: &mut V, block: &Block, context: &VisitContext) {
    for pipeline in &block.pipelines {
        visitor.visit_pipeline(pipeline, context);
    }
}

/// Walk through a pipeline, visiting all elements
pub fn walk_pipeline<V: AstVisitor + ?Sized>(
    visitor: &mut V,
    pipeline: &Pipeline,
    context: &VisitContext,
) {
    for element in &pipeline.elements {
        visitor.visit_expression(&element.expr, context);
    }
}

/// Walk through an expression, visiting all child nodes
pub fn walk_expression<V: AstVisitor + ?Sized>(
    visitor: &mut V,
    expr: &Expression,
    context: &VisitContext,
) {
    match &expr.expr {
        Expr::VarDecl(var_id) => {
            visitor.visit_var_decl(*var_id, expr.span, context);
        }
        Expr::Var(var_id) => {
            visitor.visit_var_ref(*var_id, expr.span, context);
        }
        Expr::Call(call) => {
            visitor.visit_call(call, context);
        }
        Expr::BinaryOp(lhs, op, rhs) => {
            visitor.visit_binary_op(lhs, op, rhs, context);
        }
        Expr::UnaryNot(inner) => {
            visitor.visit_expression(inner, context);
        }
        Expr::List(items) => {
            visitor.visit_list(items, context);
        }
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
        Expr::FullCellPath(cell_path) => {
            visitor.visit_expression(&cell_path.head, context);
        }
        Expr::String(content) | Expr::RawString(content) => {
            visitor.visit_string(content, expr.span, context);
        }
        Expr::Int(value) => {
            visitor.visit_int(*value, expr.span, context);
        }
        Expr::StringInterpolation(exprs) => {
            for expr in exprs {
                visitor.visit_expression(expr, context);
            }
        }
        Expr::MatchBlock(arms) => {
            for (pattern, expr) in arms {
                let _ = pattern; // TODO: implement pattern visiting
                visitor.visit_expression(expr, context);
            }
        }
        // Handle other expression types as needed
        _ => {}
    }
}

/// Walk through a function call, visiting arguments
pub fn walk_call<V: AstVisitor + ?Sized>(
    visitor: &mut V,
    call: &nu_protocol::ast::Call,
    context: &VisitContext,
) {
    // Visit positional arguments
    for expr in &call.arguments {
        match expr {
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

/// Collect all expressions of a specific type from an AST
pub struct ExpressionCollector<F> {
    pub expressions: Vec<(Expression, Span)>,
    pub predicate: F,
}

impl<F> ExpressionCollector<F>
where
    F: Fn(&Expression) -> bool,
{
    pub fn new(predicate: F) -> Self {
        Self {
            expressions: Vec::new(),
            predicate,
        }
    }
}

impl<F> AstVisitor for ExpressionCollector<F>
where
    F: Fn(&Expression) -> bool,
{
    fn visit_expression(&mut self, expr: &Expression, context: &VisitContext) {
        if (self.predicate)(expr) {
            self.expressions.push((expr.clone(), expr.span));
        }
        walk_expression(self, expr, context);
    }
}

/// Collect all variable declarations from an AST
pub struct VarDeclCollector {
    pub var_decls: Vec<(VarId, Span)>,
}

impl VarDeclCollector {
    #[must_use]
    pub fn new() -> Self {
        Self {
            var_decls: Vec::new(),
        }
    }
}

impl Default for VarDeclCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl AstVisitor for VarDeclCollector {
    fn visit_var_decl(&mut self, var_id: VarId, span: Span, _context: &VisitContext) {
        self.var_decls.push((var_id, span));
    }
}

/// Collect all function calls from an AST
pub struct CallCollector {
    pub calls: Vec<(nu_protocol::ast::Call, Span)>,
}

impl CallCollector {
    #[must_use]
    pub fn new() -> Self {
        Self { calls: Vec::new() }
    }
}

impl Default for CallCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl AstVisitor for CallCollector {
    fn visit_call(&mut self, call: &nu_protocol::ast::Call, context: &VisitContext) {
        self.calls.push((call.clone(), call.span()));
        walk_call(self, call, context);
    }
}
