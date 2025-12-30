use nu_protocol::{
    BlockId, Span, Type, VarId,
    ast::{
        Argument, Call, Expr, Expression, FindMapResult, FullCellPath, ListItem, Operator,
        PathMember, RecordItem, Traverse,
    },
    engine::Variable,
};

use super::{block::BlockExt, call::CallExt, pipeline::PipelineExt};
use crate::{
    context::LintContext,
    effect::external::{ExternEffect, has_external_side_effect},
};

pub trait ExpressionExt: Traverse {
    /// Extracts the variable name from an expression. Example: `$counter`
    /// returns "counter"
    fn extract_variable_name(&self, context: &LintContext) -> Option<String>;
    /// Checks if expression is an assignment operation. Example: `$x = 5` or
    /// `$x += 1`
    fn is_assignment(&self) -> bool;
    /// Checks if expression is an empty list literal. Example: `[]`
    fn is_empty_list(&self) -> bool;
    /// Extracts block ID from block-like expressions. Example: `{ $in | length
    /// }` or closure
    fn extract_block_id(&self) -> Option<BlockId>;
    /// Returns the source text of this expression's span. Example: `$in | each
    /// { ... }`
    fn span_text<'a>(&self, context: &'a LintContext) -> &'a str;
    /// Extracts the variable being assigned to. Example: `$result = 42` returns
    /// `$result`
    fn extract_assigned_variable(&self) -> Option<VarId>;
    /// Extracts field access from full cell path. Example: `$record.name`
    /// matches "name"
    fn extract_field_access(&self, field_name: &str) -> Option<(VarId, Span)>;

    /// Checks if expression contains any variable references. Example: `$x +
    /// $y` or `[$item]`
    fn contains_variables(&self, context: &LintContext) -> bool;
    /// Checks if external call head uses a variable. Example: `^$cmd arg1 arg2`
    fn is_external_call_with_variable(&self, var_id: VarId) -> bool;
    /// Checks if expression matches a specific variable. Example: `$var` or
    /// `$var.field`
    fn matches_var(&self, var_id: VarId) -> bool;
    /// Checks if expression contains a specific variable. Example: `$x + 1`
    /// contains `$x`
    fn contains_variable(&self, var_id: VarId) -> bool;
    /// Finds the first usage span of a specific variable. Example: `$x + 1`
    /// with `var_id` of x returns span of `$x`
    fn find_var_usage(&self, var_id: VarId) -> Option<Span>;

    /// Checks if expression uses pipeline input variable. Example: `$in` or
    /// `$in | length`
    fn uses_pipeline_input(&self, context: &LintContext) -> bool;
    /// Finds pipeline input-like variables (includes `$in` and closure
    /// parameters) and their spans. Used for type inference. Example:
    /// `$in.field` returns (`var_id`, span of `$in`)
    fn find_pipeline_input(&self, context: &LintContext) -> Option<(VarId, Span)>;
    /// Finds the actual `$in` variable usage and its span. Example: `$in.field`
    /// returns span of `$in`. Does not match closure parameters.
    fn find_dollar_in_usage(&self) -> Option<Span>;
    /// Infers the output type of an expression. Example: `ls` returns "table",
    /// `1 + 2` returns "int"
    fn infer_output_type(&self, context: &LintContext) -> Option<Type>;
    /// Infers the input type expected by an expression. Example: `$in | length`
    /// expects "list"
    fn infer_input_type(&self, in_var: Option<VarId>, context: &LintContext) -> Option<Type>;
    /// Extracts external command name from expression. Example: `^ls` returns
    /// "ls"
    fn extract_external_command_name(&self, context: &LintContext) -> Option<String>;
}

pub const fn is_dollar_in_var(var_id: VarId) -> bool {
    use nu_protocol::IN_VARIABLE_ID;
    var_id.get() == IN_VARIABLE_ID.get()
}

const fn has_synthetic_declaration_span(var: &Variable) -> bool {
    (var.declaration_span.start == 0 && var.declaration_span.end == 0)
        || (var.declaration_span.start == var.declaration_span.end
            && var.declaration_span.start > 0)
}

pub fn is_pipeline_input_var(var_id: VarId, context: &LintContext) -> bool {
    use nu_protocol::{ENV_VARIABLE_ID, NU_VARIABLE_ID};

    if is_dollar_in_var(var_id) {
        return true;
    }

    if var_id == ENV_VARIABLE_ID || var_id == NU_VARIABLE_ID {
        return false;
    }

    let var = context.working_set.get_variable(var_id);
    has_synthetic_declaration_span(var)
}

const fn extract_var_from_full_cell_path(cell_path: &FullCellPath) -> Option<VarId> {
    match &cell_path.head.expr {
        Expr::Var(var_id) => Some(*var_id),
        _ => None,
    }
}

impl ExpressionExt for Expression {
    fn extract_variable_name(&self, context: &LintContext) -> Option<String> {
        match &self.expr {
            Expr::Var(var_id) | Expr::VarDecl(var_id) => {
                let var = context.working_set.get_variable(*var_id);
                Some(context.get_span_text(var.declaration_span).to_string())
            }
            Expr::FullCellPath(cell_path) => cell_path.head.extract_variable_name(context),
            _ => None,
        }
    }

    fn is_assignment(&self) -> bool {
        matches!(
            &self.expr,
            Expr::BinaryOp(_, op, _) if matches!(
                op.expr,
                Expr::Operator(Operator::Assignment(_))
            )
        )
    }

    fn is_empty_list(&self) -> bool {
        match &self.expr {
            Expr::List(list) => list.is_empty(),
            Expr::FullCellPath(cell_path) => cell_path.head.is_empty_list(),
            _ => false,
        }
    }

    fn extract_block_id(&self) -> Option<BlockId> {
        match &self.expr {
            Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
                Some(*block_id)
            }
            _ => None,
        }
    }

    fn span_text<'a>(&self, context: &'a LintContext) -> &'a str {
        context.get_span_text(self.span)
    }

    fn extract_assigned_variable(&self) -> Option<VarId> {
        let Expr::BinaryOp(lhs, _op, _rhs) = &self.expr else {
            return None;
        };

        if !self.is_assignment() {
            return None;
        }

        match &lhs.expr {
            Expr::Var(var_id) => Some(*var_id),
            Expr::FullCellPath(cell_path) => extract_var_from_full_cell_path(cell_path),
            _ => None,
        }
    }

    fn extract_field_access(&self, field_name: &str) -> Option<(VarId, Span)> {
        if let Expr::FullCellPath(cell_path) = &self.expr
            && let Some(var_id) = extract_var_from_full_cell_path(cell_path)
            && cell_path.tail.iter().any(|path_member| {
                matches!(
                    path_member,
                    PathMember::String { val, .. } if val == field_name
                )
            })
        {
            Some((var_id, self.span))
        } else {
            None
        }
    }

    fn contains_variables(&self, context: &LintContext) -> bool {
        match &self.expr {
            Expr::Var(_) | Expr::VarDecl(_) => true,

            Expr::FullCellPath(path) => path.head.contains_variables(context),

            Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
                context
                    .working_set
                    .get_block(*block_id)
                    .contains_variables(context)
            }

            Expr::BinaryOp(left, _, right) => {
                left.contains_variables(context) || right.contains_variables(context)
            }

            Expr::UnaryNot(inner) => inner.contains_variables(context),

            Expr::List(items) => items.iter().any(|item| match item {
                ListItem::Item(expr) | ListItem::Spread(_, expr) => {
                    expr.contains_variables(context)
                }
            }),

            Expr::Table(table) => {
                table
                    .columns
                    .iter()
                    .any(|col| col.contains_variables(context))
                    || table
                        .rows
                        .iter()
                        .any(|row| row.iter().any(|cell| cell.contains_variables(context)))
            }

            Expr::Record(fields) => fields.iter().any(|field| match field {
                RecordItem::Pair(key, val) => {
                    key.contains_variables(context) || val.contains_variables(context)
                }
                RecordItem::Spread(_, expr) => expr.contains_variables(context),
            }),

            Expr::Call(call) => call.arguments.iter().any(|arg| match arg {
                Argument::Positional(expr)
                | Argument::Unknown(expr)
                | Argument::Spread(expr)
                | Argument::Named((_, _, Some(expr))) => expr.contains_variables(context),
                Argument::Named(_) => false,
            }),

            _ => false,
        }
    }

    fn is_external_call_with_variable(&self, var_id: VarId) -> bool {
        let Expr::ExternalCall(head, _args) = &self.expr else {
            return false;
        };
        head.matches_var(var_id)
    }

    fn matches_var(&self, var_id: VarId) -> bool {
        match &self.expr {
            Expr::Var(id) => *id == var_id,
            Expr::FullCellPath(cell_path) => {
                extract_var_from_full_cell_path(cell_path) == Some(var_id)
            }
            _ => false,
        }
    }

    fn contains_variable(&self, var_id: VarId) -> bool {
        match &self.expr {
            Expr::Var(id) => *id == var_id,
            Expr::FullCellPath(cell_path) => cell_path.head.contains_variable(var_id),
            Expr::BinaryOp(left, _op, right) => {
                left.contains_variable(var_id) || right.contains_variable(var_id)
            }
            Expr::UnaryNot(inner) => inner.contains_variable(var_id),
            Expr::Call(call) => call.arguments.iter().any(|arg| match arg {
                Argument::Positional(expr)
                | Argument::Named((_, _, Some(expr)))
                | Argument::Unknown(expr)
                | Argument::Spread(expr) => expr.contains_variable(var_id),
                Argument::Named(_) => false,
            }),
            Expr::List(items) => items.iter().any(|item| {
                let expr = match item {
                    ListItem::Item(e) | ListItem::Spread(_, e) => e,
                };
                expr.contains_variable(var_id)
            }),
            Expr::Table(table) => {
                table
                    .columns
                    .iter()
                    .any(|col| col.contains_variable(var_id))
                    || table
                        .rows
                        .iter()
                        .any(|row| row.iter().any(|cell| cell.contains_variable(var_id)))
            }
            Expr::Record(items) => items.iter().any(|item| match item {
                RecordItem::Pair(key, val) => {
                    key.contains_variable(var_id) || val.contains_variable(var_id)
                }
                RecordItem::Spread(_, expr) => expr.contains_variable(var_id),
            }),
            _ => false,
        }
    }

    fn find_var_usage(&self, var_id: VarId) -> Option<Span> {
        match &self.expr {
            Expr::Var(id) if *id == var_id => Some(self.span),
            Expr::FullCellPath(cell_path) => cell_path.head.find_var_usage(var_id),
            Expr::BinaryOp(left, _op, right) => left
                .find_var_usage(var_id)
                .or_else(|| right.find_var_usage(var_id)),
            Expr::UnaryNot(inner) | Expr::Collect(_, inner) => inner.find_var_usage(var_id),
            Expr::Call(call) => call.arguments.iter().find_map(|arg| match arg {
                Argument::Positional(expr)
                | Argument::Named((_, _, Some(expr)))
                | Argument::Unknown(expr)
                | Argument::Spread(expr) => expr.find_var_usage(var_id),
                Argument::Named(_) => None,
            }),
            Expr::List(items) => items.iter().find_map(|item| {
                let expr = match item {
                    ListItem::Item(e) | ListItem::Spread(_, e) => e,
                };
                expr.find_var_usage(var_id)
            }),
            Expr::Table(table) => table
                .columns
                .iter()
                .find_map(|col| col.find_var_usage(var_id))
                .or_else(|| {
                    table
                        .rows
                        .iter()
                        .find_map(|row| row.iter().find_map(|cell| cell.find_var_usage(var_id)))
                }),
            Expr::Record(items) => items.iter().find_map(|item| match item {
                RecordItem::Pair(key, val) => key
                    .find_var_usage(var_id)
                    .or_else(|| val.find_var_usage(var_id)),
                RecordItem::Spread(_, expr) => expr.find_var_usage(var_id),
            }),
            Expr::StringInterpolation(items) => {
                items.iter().find_map(|item| item.find_var_usage(var_id))
            }
            _ => None,
        }
    }

    fn uses_pipeline_input(&self, context: &LintContext) -> bool {
        if matches!(&self.expr, Expr::Collect(..)) {
            return true;
        }

        self.find_pipeline_input(context)
            .is_some_and(|(var_id, _)| {
                let var = context.working_set.get_variable(var_id);
                var.const_val.is_none()
            })
    }

    fn find_pipeline_input(&self, context: &LintContext) -> Option<(VarId, Span)> {
        use super::block::BlockExt;

        match &self.expr {
            Expr::Var(var_id) if is_pipeline_input_var(*var_id, context) => {
                Some((*var_id, self.span))
            }
            Expr::FullCellPath(cell_path) => cell_path.head.find_pipeline_input(context),
            Expr::Call(call) => call.arguments.iter().find_map(|arg| match arg {
                Argument::Positional(e)
                | Argument::Unknown(e)
                | Argument::Named((_, _, Some(e)))
                | Argument::Spread(e) => e.find_pipeline_input(context),
                Argument::Named(_) => None,
            }),
            Expr::BinaryOp(lhs, _, rhs) => lhs
                .find_pipeline_input(context)
                .or_else(|| rhs.find_pipeline_input(context)),
            Expr::UnaryNot(e) | Expr::Collect(_, e) => e.find_pipeline_input(context),
            Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
                context
                    .working_set
                    .get_block(*block_id)
                    .find_pipeline_input(context)
            }
            Expr::StringInterpolation(items) => items
                .iter()
                .find_map(|item| item.find_pipeline_input(context)),
            _ => None,
        }
    }

    fn find_dollar_in_usage(&self) -> Option<Span> {
        match &self.expr {
            Expr::Var(var_id) if is_dollar_in_var(*var_id) => Some(self.span),
            Expr::FullCellPath(cell_path) => cell_path.head.find_dollar_in_usage(),
            Expr::Call(call) => call.arguments.iter().find_map(|arg| match arg {
                Argument::Positional(e)
                | Argument::Unknown(e)
                | Argument::Named((_, _, Some(e)))
                | Argument::Spread(e) => e.find_dollar_in_usage(),
                Argument::Named(_) => None,
            }),
            Expr::BinaryOp(lhs, _, rhs) => lhs
                .find_dollar_in_usage()
                .or_else(|| rhs.find_dollar_in_usage()),
            Expr::UnaryNot(e) | Expr::Collect(_, e) => e.find_dollar_in_usage(),
            Expr::StringInterpolation(items) => {
                items.iter().find_map(ExpressionExt::find_dollar_in_usage)
            }
            _ => None,
        }
    }

    fn infer_output_type(&self, context: &LintContext) -> Option<Type> {
        log::debug!(
            "Inferring output type for expression: '{}'",
            self.span_text(context)
        );

        let inner_expr = match &self.expr {
            Expr::Collect(_, inner) => {
                log::debug!(
                    "Replacing Expr::Collect with inner expression: '{}'",
                    inner.span_text(context)
                );
                &inner.expr
            }
            _ => &self.expr,
        };

        log::debug!("About to match against expression variant");

        match inner_expr {
            expr if check_filepath_output(expr).is_some() => check_filepath_output(expr),
            Expr::Bool(_)
            | Expr::Int(_)
            | Expr::Float(_)
            | Expr::String(_)
            | Expr::StringInterpolation(_)
            | Expr::RawString(_)
            | Expr::Record(_)
            | Expr::Table(_) => {
                log::debug!("Matched literal expression, using AST type: {:?}", self.ty);
                Some(self.ty.clone())
            }
            Expr::BinaryOp(left, _op, right) => {
                if !matches!(self.ty, Type::Any) {
                    return Some(self.ty.clone());
                }

                infer_binary_op_type(
                    left.infer_output_type(context).as_ref(),
                    right.infer_output_type(context).as_ref(),
                )
                .or_else(|| Some(self.ty.clone()))
            }
            Expr::List(items) => {
                log::debug!("Matched List literal with {} items", items.len());
                Some(infer_list_element_type(items))
            }
            Expr::Nothing => {
                log::debug!("Matched Nothing");
                Some(Type::Nothing)
            }
            Expr::FullCellPath(path) => {
                log::debug!("Matched FullCellPath, checking head");
                if let Expr::List(items) = &path.head.expr {
                    log::debug!("FullCellPath contains List with {} items", items.len());
                    return Some(infer_list_element_type(items));
                }

                if !path.tail.is_empty() {
                    log::debug!("Using head type for FullCellPath: {:?}", path.head.ty);
                    return Some(path.head.ty.clone());
                }

                log::debug!("FullCellPath has empty tail, inferring head type recursively");
                let inferred = path.head.infer_output_type(context);
                log::debug!("FullCellPath inferred type from head: {inferred:?}");
                inferred.or_else(|| Some(path.head.ty.clone()))
            }
            Expr::Subexpression(block_id) | Expr::Block(block_id) => {
                log::debug!("Encountered Subexpression");
                Some(
                    context
                        .working_set
                        .get_block(*block_id)
                        .infer_output_type(context),
                )
            }
            Expr::ExternalCall(call, args) => {
                let cmd_name = context.get_span_text(call.span);
                log::debug!("Encountered ExternalCall: '{cmd_name}'");
                if has_external_side_effect(cmd_name, ExternEffect::NoDataInStdout, context, args) {
                    Some(Type::Nothing)
                } else {
                    Some(Type::String)
                }
            }
            Expr::Call(call) => {
                let decl = context.working_set.get_decl(call.decl_id);
                let cmd_name = decl.name();
                log::debug!("Encountered Call: '{cmd_name}'");
                if matches!(cmd_name, "if" | "match" | "try" | "do")
                    && let Some(unified_type) = call.infer_from_blocks(context)
                {
                    log::debug!(
                        "Inferred unified type from blocks for '{cmd_name}': {unified_type:?}"
                    );
                    return Some(unified_type);
                }

                Some(call.get_output_type(context, None))
            }
            _other => {
                log::debug!("No specific match, using default case");
                None
            }
        }
    }

    fn infer_input_type(&self, in_var: Option<VarId>, context: &LintContext) -> Option<Type> {
        let in_var_id = in_var?;
        log::debug!(
            "infer_input_type: checking expr='{}', var_id={in_var_id:?}",
            self.span_text(context)
        );

        let result = match &self.expr {
            Expr::FullCellPath(cell_path) if matches!(&cell_path.head.expr, Expr::Var(var_id) if *var_id == in_var_id) =>
            {
                log::debug!(
                    "  -> FullCellPath with matching var, tail_len={}",
                    cell_path.tail.len()
                );
                if !cell_path.tail.is_empty()
                    && cell_path
                        .tail
                        .iter()
                        .any(|member| matches!(member, PathMember::String { .. }))
                {
                    Some(Type::Record(Box::new([])))
                } else if !cell_path.tail.is_empty() {
                    Some(Type::List(Box::new(Type::Any)))
                } else {
                    None
                }
            }
            Expr::FullCellPath(cell_path)
                if matches!(
                    &cell_path.head.expr,
                    Expr::Subexpression(_) | Expr::Block(_) | Expr::Closure(_)
                ) =>
            {
                log::debug!("  -> FullCellPath wrapping block-like expression");
                cell_path.head.infer_input_type(in_var, context)
            }
            Expr::Call(call) => {
                log::debug!("  -> Call expression, checking arguments");
                infer_from_call(call, in_var_id, in_var, context)
            }
            Expr::BinaryOp(left, op_expr, right) => {
                log::debug!("  -> BinaryOp, checking if math/comparison with variable");
                if matches!(&op_expr.expr, Expr::Operator(op) if matches!(op, Operator::Math(_) | Operator::Comparison(_)))
                    && (left.contains_variable(in_var_id) || right.contains_variable(in_var_id))
                {
                    log::debug!("  -> Found math/comparison op with variable, returning Int");
                    return Some(Type::Int);
                }

                log::debug!("  -> Recursing into BinaryOp operands");
                left.infer_input_type(in_var, context)
                    .or_else(|| right.infer_input_type(in_var, context))
            }
            Expr::Collect(_, inner) | Expr::UnaryNot(inner) => {
                log::debug!("  -> Collect/UnaryNot, checking inner");
                inner.infer_input_type(in_var, context)
            }
            Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
                log::debug!("  -> Subexpression/Block/Closure, block_id={block_id:?}");
                let block = context.working_set.get_block(*block_id);
                log::debug!("     Block has {} pipelines", block.pipelines.len());

                let pipeline_type = block
                    .pipelines
                    .iter()
                    .find_map(|pipeline| pipeline.infer_param_type(in_var_id, context));

                if pipeline_type.is_some() {
                    log::debug!("     Found type from pipeline analysis: {pipeline_type:?}");
                    return pipeline_type;
                }

                block
                    .pipelines
                    .iter()
                    .flat_map(|pipeline| &pipeline.elements)
                    .find_map(|element| {
                        log::debug!(
                            "    -> Checking pipeline element, expr='{}', variant={:?}",
                            element.expr.span_text(context),
                            &element.expr.expr
                        );
                        let result = element.expr.infer_input_type(in_var, context);
                        log::debug!("       Result: {result:?}");
                        result
                    })
            }
            Expr::MatchBlock(patterns) => {
                log::debug!("  -> MatchBlock, checking patterns");
                patterns
                    .iter()
                    .find_map(|(_, expr)| expr.infer_input_type(in_var, context))
            }
            _ => {
                log::debug!("  -> No match for expression type");
                None
            }
        };

        log::debug!("infer_input_type result: {result:?}");
        result
    }

    fn extract_external_command_name(&self, context: &LintContext) -> Option<String> {
        use nu_protocol::ast::Traverse;

        self.find_map(context.working_set, &|inner_expr| {
            if let Expr::ExternalCall(cmd_expr, _) = &inner_expr.expr {
                match &cmd_expr.expr {
                    Expr::String(s) => FindMapResult::Found(s.clone()),
                    Expr::GlobPattern(pattern, _) => FindMapResult::Found(pattern.clone()),
                    _ => FindMapResult::Continue,
                }
            } else {
                FindMapResult::Continue
            }
        })
    }
}

fn infer_from_call(
    call: &Call,
    in_var_id: VarId,
    in_var: Option<VarId>,
    context: &LintContext,
) -> Option<Type> {
    log::debug!("infer_from_call: checking call for var_id={in_var_id:?}");

    for (idx, arg) in call.arguments.iter().enumerate() {
        if let Argument::Positional(arg_expr) | Argument::Unknown(arg_expr) = arg {
            log::debug!("  -> Checking positional arg {idx}");
            if !arg_expr.contains_variable(in_var_id) {
                log::debug!("    -> Does not contain variable");
                continue;
            }

            log::debug!("    -> Contains variable! Checking signature");
            let decl = context.working_set.get_decl(call.decl_id);
            let signature = decl.signature();

            log::debug!(
                "    -> Command: '{}', input_output_types: {:?}",
                decl.name(),
                signature.input_output_types
            );

            if let Some((input_type, _)) = signature.input_output_types.first()
                && !matches!(input_type, nu_protocol::Type::Any)
            {
                log::debug!("    -> Found input type from signature: {input_type:?}");
                return Some(input_type.clone());
            }
            log::debug!("    -> Signature has no specific input type");
        }
    }

    log::debug!("  -> Recursively checking call arguments");
    let result = call.arguments.iter().find_map(|arg| match arg {
        Argument::Positional(arg_expr) | Argument::Unknown(arg_expr) => {
            arg_expr.infer_input_type(in_var, context)
        }
        _ => None,
    });

    log::debug!("infer_from_call result: {result:?}");
    result
}

const fn is_filepath_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Filepath(..))
}

const fn is_glob_pattern_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::GlobPattern(..))
}

fn check_filepath_output(expr: &Expr) -> Option<Type> {
    let ty = Type::Custom("path".into());
    match expr {
        Expr::ExternalCall(head, _) if matches!(&head.expr, Expr::Filepath(..)) => {
            log::debug!(
                "check_filepath_output: ExternalCall with filepath head: {:?}",
                head.expr
            );
            Some(ty)
        }
        Expr::Collect(_, inner) if is_filepath_expr(&inner.expr) => {
            log::debug!("check_filepath_output: Collect with filepath inner");
            Some(ty)
        }
        expr if is_filepath_expr(expr) || is_glob_pattern_expr(expr) => {
            log::debug!("check_filepath_output: filepath expr: {expr:?}");
            Some(ty)
        }
        _ => None,
    }
}

const fn infer_binary_op_type(left: Option<&Type>, right: Option<&Type>) -> Option<Type> {
    match (left, right) {
        (Some(Type::Float), _) | (_, Some(Type::Float)) => Some(Type::Float),
        (Some(Type::Int | Type::Any), Some(Type::Int)) | (Some(Type::Int), Some(Type::Any)) => {
            Some(Type::Int)
        }
        _ => None,
    }
}

fn infer_list_element_type(items: &[ListItem]) -> Type {
    if items.is_empty() {
        return Type::List(Box::new(Type::Any));
    }

    let element_types: Vec<Type> = items
        .iter()
        .map(|item| match item {
            ListItem::Item(expr) | ListItem::Spread(_, expr) => expr.ty.clone(),
        })
        .collect();

    if element_types.is_empty() {
        return Type::List(Box::new(Type::Any));
    }

    if element_types.iter().all(|t| t == &element_types[0]) {
        log::debug!("All list elements have type: {:?}", element_types[0]);
        Type::List(Box::new(element_types[0].clone()))
    } else {
        log::debug!("List has mixed types, using Any");
        Type::List(Box::new(Type::Any))
    }
}
