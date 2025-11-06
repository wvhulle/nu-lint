use nu_protocol::{
    BlockId, Span, VarId,
    ast::{
        Argument, Call, Comparison, Expr, Expression, ExternalArgument, FindMapResult, ListItem,
        Operator, PathMember, RecordItem, Traverse,
    },
};

use super::{block::BlockExt, span::SpanExt};
use crate::context::LintContext;

pub trait ExpressionExt: Traverse {
    fn refers_to_same_variable(&self, other: &Expression, context: &LintContext) -> bool;
    fn extract_variable_name(&self, context: &LintContext) -> Option<String>;
    fn refers_to_variable(&self, context: &LintContext, var_name: &str) -> bool;
    fn is_assignment(&self) -> bool;
    fn is_empty_list(&self) -> bool;
    fn extract_block_id(&self) -> Option<BlockId>;
    fn is_likely_pure(&self) -> bool;
    fn span_text<'a>(&self, context: &'a LintContext) -> &'a str;
    fn extract_assigned_variable(&self) -> Option<VarId>;
    fn extract_field_access(&self, field_name: &str) -> Option<(VarId, Span)>;

    fn contains_variables(&self, context: &LintContext) -> bool;
    fn extract_compared_variable(&self, context: &LintContext) -> Option<String>;
    fn extract_comparison_value(&self, context: &LintContext) -> Option<String>;
    fn is_external_call_with_variable(&self, var_id: VarId) -> bool;
    fn matches_var(&self, var_id: VarId) -> bool;
    fn external_call_contains_variable(&self, var_id: VarId) -> bool;
    fn is_external_filesystem_command(&self, context: &LintContext) -> bool;
    fn extract_call(&self) -> Option<&Call>;
    fn contains_variable(&self, var_id: VarId) -> bool;

    #[allow(dead_code, reason = "Will be used later.")]
    fn any(&self, context: &LintContext, predicate: impl Fn(&Expression) -> bool) -> bool;
    fn uses_pipeline_input(&self, context: &LintContext) -> bool;
    /// Finds the `$in` variable in this expression. Example: `$in.field` or
    /// `$in | length`
    fn find_pipeline_input_variable(&self, context: &LintContext) -> Option<VarId>;
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
            Expr::FullCellPath(cell_path) => {
                if let Expr::Var(var_id) = &cell_path.head.expr {
                    Some(*var_id)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn extract_field_access(&self, field_name: &str) -> Option<(VarId, Span)> {
        if let Expr::FullCellPath(cell_path) = &self.expr
            && let Expr::Var(var_id) = &cell_path.head.expr
            && cell_path.tail.iter().any(|path_member| {
                matches!(
                    path_member,
                    PathMember::String { val, .. } if val == field_name
                )
            })
        {
            Some((*var_id, self.span))
        } else {
            None
        }
    }

    fn contains_variables(&self, context: &LintContext) -> bool {
        match &self.expr {
            Expr::Var(_) | Expr::VarDecl(_) => true,

            Expr::FullCellPath(path) => path.head.contains_variables(context),

            Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
                block_id.contains_variables(context)
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
                matches!(&cell_path.head.expr, Expr::Var(id) if *id == var_id)
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
        const EXTERNAL_FILESYSTEM_COMMANDS: &[&str] =
            &["tar", "zip", "unzip", "rsync", "scp", "wget", "curl"];

        if let Expr::ExternalCall(head, _) = &self.expr {
            let cmd_name = &context.source[head.span.start..head.span.end];
            let lower_cmd = cmd_name.to_lowercase();
            EXTERNAL_FILESYSTEM_COMMANDS
                .iter()
                .any(|&cmd| lower_cmd == cmd)
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
                let span_start = var.declaration_span.start;
                let span_end = var.declaration_span.end;
                var.const_val.is_none() && span_start == 0 && span_end == 0
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
            Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
                block_id.uses_pipeline_input(context)
            }
            _ => false,
        }
    }

    fn find_pipeline_input_variable(&self, context: &LintContext) -> Option<VarId> {
        use super::block::BlockExt;

        match &self.expr {
            Expr::Var(var_id) => {
                let var = context.working_set.get_variable(*var_id);
                // $in has declaration_span (0,0) or start==end
                if (var.declaration_span.start == 0 && var.declaration_span.end == 0)
                    || (var.declaration_span.start == var.declaration_span.end
                        && var.declaration_span.start > 0)
                {
                    return Some(*var_id);
                }
                None
            }
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
                block_id.find_pipeline_input_variable(context)
            }
            Expr::StringInterpolation(items) => items
                .iter()
                .find_map(|item| item.find_pipeline_input_variable(context)),
            _ => None,
        }
    }
}
