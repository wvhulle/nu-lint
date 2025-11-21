use nu_protocol::{
    BlockId, Span, Type, VarId,
    ast::{
        Argument, Call, Comparison, Expr, Expression, ExternalArgument, FindMapResult,
        FullCellPath, ListItem, Operator, PathMember, RecordItem, Traverse,
    },
};

use super::{
    block::BlockExt, call::CallExt, ext_command::ExternalCommandExt, pipeline::PipelineExt,
    span::SpanExt,
};
use crate::context::LintContext;

pub trait ExpressionExt: Traverse {
    /// Checks if two expressions refer to the same variable. Example: `$x` and
    /// `$x`
    fn refers_to_same_variable(&self, other: &Expression, context: &LintContext) -> bool;
    /// Extracts the variable name from an expression. Example: `$counter`
    /// returns "counter"
    fn extract_variable_name(&self, context: &LintContext) -> Option<String>;
    /// Checks if expression refers to a specific variable by name. Example:
    /// `$item` matches "item"
    fn refers_to_variable(&self, context: &LintContext, var_name: &str) -> bool;
    /// Checks if expression is an assignment operation. Example: `$x = 5` or
    /// `$x += 1`
    fn is_assignment(&self) -> bool;
    /// Checks if expression is an empty list literal. Example: `[]`
    fn is_empty_list(&self) -> bool;
    /// Extracts block ID from block-like expressions. Example: `{ $in | length
    /// }` or closure
    fn extract_block_id(&self) -> Option<BlockId>;
    /// Has no side effects: `5`, `"text"`
    fn is_likely_pure(&self) -> bool;
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
    /// Extracts variable from comparison expression. Example: `$status == 0`
    /// returns "status"
    fn extract_compared_variable(&self, context: &LintContext) -> Option<String>;
    /// Extracts comparison value from binary operation. Example: `$x ==
    /// "value"` returns "value"
    fn extract_comparison_value(&self, context: &LintContext) -> Option<String>;
    /// Checks if external call head uses a variable. Example: `^$cmd arg1 arg2`
    fn is_external_call_with_variable(&self, var_id: VarId) -> bool;
    /// Checks if expression matches a specific variable. Example: `$var` or
    /// `$var.field`
    fn matches_var(&self, var_id: VarId) -> bool;
    /// Checks if external call arguments contain a variable. Example: `^ls
    /// $path`
    fn external_call_contains_variable(&self, var_id: VarId) -> bool;
    /// Checks if external call is a filesystem command. Example: `^tar`,
    /// `^rsync`, or `^curl`
    fn is_external_filesystem_command(&self, context: &LintContext) -> bool;
    /// Extracts Call from a call expression. Example: `ls | where size > 1kb`
    fn extract_call(&self) -> Option<&Call>;
    /// Checks if expression contains a specific variable. Example: `$x + 1`
    /// contains `$x`
    fn contains_variable(&self, var_id: VarId) -> bool;

    #[allow(dead_code, reason = "Will be used later.")]
    /// Tests if any nested expression matches predicate. Example: finds `$in`
    /// in `$in.field + 1`
    fn any(&self, context: &LintContext, predicate: impl Fn(&Expression) -> bool) -> bool;
    /// Checks if expression uses pipeline input variable. Example: `$in` or
    /// `$in | length`
    fn uses_pipeline_input(&self, context: &LintContext) -> bool;
    /// Finds the `$in` variable in this expression. Example: `$in.field` or
    /// `$in | length`
    fn find_pipeline_input_variable(&self, context: &LintContext) -> Option<VarId>;
    /// Infers the output type of an expression. Example: `ls` returns "table",
    /// `1 + 2` returns "int"
    fn infer_output_type(&self, context: &LintContext) -> Option<Type>;
    /// Infers the input type expected by an expression. Example: `$in | length`
    /// expects "list"
    fn infer_input_type(&self, in_var: Option<VarId>, context: &LintContext) -> Option<Type>;
    /// Checks if expression is a literal list. Example: `[1 2 3]` or `[]`
    fn is_literal_list(&self) -> bool;
    /// Extracts external command name from expression. Example: `^ls` returns
    /// "ls"
    fn extract_external_command_name(&self, context: &LintContext) -> Option<String>;
}

fn is_pipeline_input_var(var_id: VarId, context: &LintContext) -> bool {
    let var = context.working_set.get_variable(var_id);
    (var.declaration_span.start == 0 && var.declaration_span.end == 0)
        || (var.declaration_span.start == var.declaration_span.end
            && var.declaration_span.start > 0)
}

const fn extract_var_from_full_cell_path(cell_path: &FullCellPath) -> Option<VarId> {
    match &cell_path.head.expr {
        Expr::Var(var_id) => Some(*var_id),
        _ => None,
    }
}

impl ExpressionExt for Expression {
    fn any(&self, context: &LintContext, predicate: impl Fn(&Self) -> bool) -> bool {
        self.find_map(context.working_set, &|inner_expr| {
            if predicate(inner_expr) {
                return FindMapResult::Found(());
            }
            FindMapResult::Continue
        })
        .is_some()
    }
    fn refers_to_same_variable(&self, other: &Expression, context: &LintContext) -> bool {
        match (
            self.extract_variable_name(context),
            other.extract_variable_name(context),
        ) {
            (Some(name1), Some(name2)) => name1 == name2,
            _ => false,
        }
    }

    fn extract_variable_name(&self, context: &LintContext) -> Option<String> {
        match &self.expr {
            Expr::Var(var_id) | Expr::VarDecl(var_id) => {
                let var = context.working_set.get_variable(*var_id);
                Some(var.declaration_span.text(context).to_string())
            }
            Expr::FullCellPath(cell_path) => cell_path.head.extract_variable_name(context),
            _ => None,
        }
    }

    fn refers_to_variable(&self, context: &LintContext, var_name: &str) -> bool {
        self.extract_variable_name(context)
            .is_some_and(|name| name == var_name)
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

    fn is_likely_pure(&self) -> bool {
        match &self.expr {
            Expr::Bool(_)
            | Expr::Int(_)
            | Expr::Float(_)
            | Expr::Binary(_)
            | Expr::String(_)
            | Expr::RawString(_)
            | Expr::Filepath(_, _)
            | Expr::Directory(_, _)
            | Expr::GlobPattern(_, _)
            | Expr::List(_)
            | Expr::Record(_)
            | Expr::Table(_)
            | Expr::Keyword(_)
            | Expr::Nothing
            | Expr::ValueWithUnit(_)
            | Expr::DateTime(_)
            | Expr::Range(_)
            | Expr::Var(_)
            | Expr::VarDecl(_)
            | Expr::FullCellPath(_) => true,

            Expr::BinaryOp(left, _op, right) => {
                if self.is_assignment() {
                    return false;
                }
                left.is_likely_pure() && right.is_likely_pure()
            }

            Expr::UnaryNot(inner) => inner.is_likely_pure(),

            _ => false,
        }
    }

    fn span_text<'a>(&self, context: &'a LintContext) -> &'a str {
        &context.source[self.span.start..self.span.end]
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

            Expr::Record(fields) => fields.iter().any(|field| match field {
                RecordItem::Pair(key, val) => {
                    key.contains_variables(context) || val.contains_variables(context)
                }
                RecordItem::Spread(_, expr) => expr.contains_variables(context),
            }),

            Expr::Call(call) => call.arguments.iter().any(|arg| match arg {
                Argument::Positional(expr)
                | Argument::Unknown(expr)
                | Argument::Named((_, _, Some(expr))) => expr.contains_variables(context),
                _ => false,
            }),

            _ => false,
        }
    }

    fn extract_compared_variable(&self, context: &LintContext) -> Option<String> {
        let Expr::BinaryOp(left, op, _right) = &self.expr else {
            return None;
        };

        let Expr::Operator(Operator::Comparison(Comparison::Equal | Comparison::NotEqual)) =
            &op.expr
        else {
            return None;
        };

        if let Some(var_name) = left.extract_variable_name(context) {
            return Some(var_name);
        }

        if let Expr::FullCellPath(cell_path) = &left.expr {
            Some(cell_path.head.span.text(context).to_string())
        } else {
            None
        }
    }

    fn extract_comparison_value(&self, context: &LintContext) -> Option<String> {
        let Expr::BinaryOp(_left, _op, right) = &self.expr else {
            return None;
        };

        Some(right.span_text(context).to_string())
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

    fn external_call_contains_variable(&self, var_id: VarId) -> bool {
        if let Expr::ExternalCall(_head, args) = &self.expr {
            args.iter().any(|arg| {
                let arg_expr = match arg {
                    ExternalArgument::Regular(e) | ExternalArgument::Spread(e) => e,
                };
                arg_expr.matches_var(var_id)
            })
        } else {
            false
        }
    }

    fn is_external_filesystem_command(&self, context: &LintContext) -> bool {
        use crate::ast::effect::{IoType, get_external_io_type};

        if let Expr::ExternalCall(head, _) = &self.expr {
            let cmd_name = &context.source[head.span.start..head.span.end];
            matches!(get_external_io_type(cmd_name), Some(IoType::FileSystem))
        } else {
            false
        }
    }

    fn extract_call(&self) -> Option<&Call> {
        match &self.expr {
            Expr::Call(call) => Some(call),
            _ => None,
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

    fn uses_pipeline_input(&self, context: &LintContext) -> bool {
        use super::block::BlockExt;

        match &self.expr {
            Expr::Var(var_id) => {
                let var = context.working_set.get_variable(*var_id);
                is_pipeline_input_var(*var_id, context) && var.const_val.is_none()
            }
            Expr::BinaryOp(left, _, right) => {
                left.uses_pipeline_input(context) || right.uses_pipeline_input(context)
            }
            Expr::UnaryNot(inner) => inner.uses_pipeline_input(context),
            Expr::Collect(_var_id, _inner_expr) => true,
            Expr::Call(call) => call.arguments.iter().any(|arg| {
                if let Argument::Positional(arg_expr) | Argument::Named((_, _, Some(arg_expr))) =
                    arg
                {
                    arg_expr.uses_pipeline_input(context)
                } else {
                    false
                }
            }),
            Expr::FullCellPath(cell_path) => cell_path.head.uses_pipeline_input(context),
            Expr::Subexpression(block_id) | Expr::Block(block_id) => context
                .working_set
                .get_block(*block_id)
                .uses_pipeline_input(context),
            _ => false,
        }
    }

    fn find_pipeline_input_variable(&self, context: &LintContext) -> Option<VarId> {
        use super::block::BlockExt;

        match &self.expr {
            Expr::Var(var_id) => is_pipeline_input_var(*var_id, context).then_some(*var_id),
            Expr::FullCellPath(cell_path) => cell_path.head.find_pipeline_input_variable(context),
            Expr::Call(call) => call.arguments.iter().find_map(|arg| match arg {
                Argument::Positional(e)
                | Argument::Unknown(e)
                | Argument::Named((_, _, Some(e)))
                | Argument::Spread(e) => e.find_pipeline_input_variable(context),
                Argument::Named(_) => None,
            }),
            Expr::BinaryOp(lhs, _, rhs) => lhs
                .find_pipeline_input_variable(context)
                .or_else(|| rhs.find_pipeline_input_variable(context)),
            Expr::UnaryNot(e) | Expr::Collect(_, e) => e.find_pipeline_input_variable(context),
            Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
                context
                    .working_set
                    .get_block(*block_id)
                    .find_pipeline_input_variable(context)
            }
            Expr::StringInterpolation(items) => items
                .iter()
                .find_map(|item| item.find_pipeline_input_variable(context)),
            _ => None,
        }
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Type inference requires many match arms"
    )]
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
            Expr::ExternalCall(call, _) => {
                let cmd_name = call.span.text(context);
                log::debug!("Encountered ExternalCall: '{cmd_name}'");
                if cmd_name.is_known_external_no_output_command() {
                    Some(Type::Nothing)
                } else if cmd_name.is_known_external_output_command() {
                    Some(Type::String)
                } else {
                    None
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

    fn is_literal_list(&self) -> bool {
        match &self.expr {
            Expr::List(_) => true,
            Expr::FullCellPath(cell_path) => matches!(&cell_path.head.expr, Expr::List(_)),
            Expr::Keyword(keyword) => keyword.expr.is_literal_list(),
            _ => false,
        }
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
    matches!(expr, Expr::Filepath(..) | Expr::GlobPattern(..))
}

fn check_filepath_output(expr: &Expr) -> Option<Type> {
    let ty = Type::Custom("path".into());
    match expr {
        Expr::ExternalCall(head, _) if is_filepath_expr(&head.expr) => Some(ty),
        Expr::Collect(_, inner) if is_filepath_expr(&inner.expr) => Some(ty),
        expr if is_filepath_expr(expr) => Some(ty),
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
