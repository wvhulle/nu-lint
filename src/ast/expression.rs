use nu_protocol::{
    BlockId, Span, VarId,
    ast::{Expr, Expression, Operator, PathMember},
};

use super::{BlockExt, CallExt, SpanExt};
use crate::context::LintContext;

#[must_use]
fn accesses_field(path_tail: &[PathMember], field_name: &str) -> bool {
    path_tail.iter().any(|path_member| {
        matches!(
            path_member,
            PathMember::String { val, .. } if val == field_name
        )
    })
}

pub trait ExpressionExt {
    fn refers_to_same_variable(&self, other: &Expression, context: &LintContext) -> bool;
    fn extract_variable_name(&self, context: &LintContext) -> Option<String>;
    fn refers_to_variable(&self, context: &LintContext, var_name: &str) -> bool;
    fn is_assignment(&self) -> bool;
    fn is_empty_list(&self) -> bool;
    fn extract_block_id(&self) -> Option<BlockId>;
    fn is_likely_pure(&self) -> bool;
    fn span_text<'a>(&self, context: &'a LintContext) -> &'a str;
    fn is_call_to(&self, command_name: &str, context: &LintContext) -> bool;
    fn extract_assigned_variable(&self) -> Option<VarId>;
    fn extract_field_access(&self, field_name: &str) -> Option<(VarId, Span)>;
    fn is_external_call(&self) -> bool;
    fn extract_external_command_name(&self, context: &LintContext) -> Option<String>;
    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool;
    fn contains_variables(&self, context: &LintContext) -> bool;
    fn extract_compared_variable(&self, context: &LintContext) -> Option<String>;
    fn extract_comparison_value(&self, context: &LintContext) -> Option<String>;
    fn is_external_call_with_variable(&self, var_id: VarId) -> bool;
    fn matches_var(&self, var_id: VarId) -> bool;
    fn external_call_contains_variable(&self, var_id: VarId) -> bool;
    fn is_external_filesystem_command(&self, context: &LintContext) -> bool;
    fn extract_call(&self) -> Option<&nu_protocol::ast::Call>;
    fn contains_variable(&self, var_id: VarId) -> bool;
}

impl ExpressionExt for Expression {
    fn refers_to_same_variable(&self, other: &Expression, context: &LintContext) -> bool {
        let text1 = self.span_text(context);
        let text2 = other.span_text(context);
        text1 == text2
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
        if let Some(name) = self.extract_variable_name(context) {
            name == var_name
        } else {
            false
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
                // Assignment is not pure
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

    fn is_call_to(&self, command_name: &str, context: &LintContext) -> bool {
        match &self.expr {
            Expr::Call(call) => call.is_call_to_command(command_name, context),
            _ => false,
        }
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
            && accesses_field(&cell_path.tail, field_name)
        {
            Some((*var_id, self.span))
        } else {
            None
        }
    }

    fn is_external_call(&self) -> bool {
        matches!(&self.expr, Expr::ExternalCall(_, _))
    }

    fn extract_external_command_name(&self, context: &LintContext) -> Option<String> {
        if let Expr::ExternalCall(head, _args) = &self.expr {
            Some(head.span.text(context).to_string())
        } else {
            None
        }
    }

    fn contains_call_to(&self, command_name: &str, context: &LintContext) -> bool {
        use nu_protocol::ast::Traverse;

        let mut results = Vec::new();
        self.flat_map(
            context.working_set,
            &|inner_expr| {
                if inner_expr.is_call_to(command_name, context) {
                    vec![true]
                } else {
                    vec![]
                }
            },
            &mut results,
        );
        !results.is_empty()
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
                nu_protocol::ast::ListItem::Item(expr)
                | nu_protocol::ast::ListItem::Spread(_, expr) => expr.contains_variables(context),
            }),

            Expr::Record(fields) => fields.iter().any(|field| match field {
                nu_protocol::ast::RecordItem::Pair(key, val) => {
                    key.contains_variables(context) || val.contains_variables(context)
                }
                nu_protocol::ast::RecordItem::Spread(_, expr) => expr.contains_variables(context),
            }),

            Expr::Call(call) => call.arguments.iter().any(|arg| match arg {
                nu_protocol::ast::Argument::Positional(expr)
                | nu_protocol::ast::Argument::Unknown(expr)
                | nu_protocol::ast::Argument::Named((_, _, Some(expr))) => {
                    expr.contains_variables(context)
                }
                _ => false,
            }),

            _ => false,
        }
    }

    fn extract_compared_variable(&self, context: &LintContext) -> Option<String> {
        let Expr::BinaryOp(left, op, _right) = &self.expr else {
            return None;
        };

        let Expr::Operator(Operator::Comparison(
            nu_protocol::ast::Comparison::Equal | nu_protocol::ast::Comparison::NotEqual,
        )) = &op.expr
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

        match &head.expr {
            Expr::Var(id) => *id == var_id,
            Expr::FullCellPath(cell_path) => {
                matches!(&cell_path.head.expr, Expr::Var(id) if *id == var_id)
            }
            _ => false,
        }
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
                    nu_protocol::ast::ExternalArgument::Regular(e)
                    | nu_protocol::ast::ExternalArgument::Spread(e) => e,
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

    fn extract_call(&self) -> Option<&nu_protocol::ast::Call> {
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
                nu_protocol::ast::Argument::Positional(expr)
                | nu_protocol::ast::Argument::Named((_, _, Some(expr)))
                | nu_protocol::ast::Argument::Unknown(expr)
                | nu_protocol::ast::Argument::Spread(expr) => expr.contains_variable(var_id),
                nu_protocol::ast::Argument::Named(_) => false,
            }),
            Expr::List(items) => items.iter().any(|item| {
                let expr = match item {
                    nu_protocol::ast::ListItem::Item(e)
                    | nu_protocol::ast::ListItem::Spread(_, e) => e,
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
                nu_protocol::ast::RecordItem::Pair(key, val) => {
                    key.contains_variable(var_id) || val.contains_variable(var_id)
                }
                nu_protocol::ast::RecordItem::Spread(_, expr) => expr.contains_variable(var_id),
            }),
            _ => false,
        }
    }
}
