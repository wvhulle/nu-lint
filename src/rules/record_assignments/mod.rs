mod merge_flat_upsert;
mod merge_nested_upsert;
mod use_load_env;

pub use merge_flat_upsert::RULE as MERGE_FLAT_UPSERT;
pub use merge_nested_upsert::RULE as MERGE_NESTED_UPSERT;
use nu_protocol::{
    Span, VarId,
    ast::{Assignment, Block, Expr, Expression, Operator, PathMember, Pipeline},
};
pub use use_load_env::RULE as USE_LOAD_ENV;

use crate::{
    ast::{
        expression::var_name_from_expr,
        string::{cell_path_member_needs_quotes, record_key_needs_quotes},
    },
    context::LintContext,
    violation::Detection,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Field(String),
    Index(i64),
}

impl PathSegment {
    fn from_path_member(member: &PathMember) -> Self {
        match member {
            PathMember::String { val, .. } => Self::Field(val.clone()),
            #[expect(
                clippy::cast_possible_wrap,
                reason = "nu-protocol uses i64 for int indices"
            )]
            PathMember::Int { val, .. } => Self::Index(*val as i64),
        }
    }

    pub fn to_record_key(&self) -> String {
        match self {
            Self::Field(name) if !record_key_needs_quotes(name) => name.clone(),
            Self::Field(name) => format!("\"{name}\""),
            Self::Index(idx) => format!("\"{idx}\""),
        }
    }
}

pub struct FieldAssignment {
    pub path: Vec<PathSegment>,
    pub full_span: Span,
    pub value_span: Span,
}

pub struct AssignmentGroup {
    pub root_var_name: String,
    pub is_env: bool,
    pub assignments: Vec<FieldAssignment>,
    pub combined_span: Span,
    pub all_flat: bool,
}

fn extract_field_assignment(
    expr: &Expression,
    context: &LintContext,
) -> Option<(VarId, String, FieldAssignment)> {
    let Expr::BinaryOp(lhs, op, rhs) = &expr.expr else {
        return None;
    };
    let Expr::Operator(Operator::Assignment(Assignment::Assign)) = &op.expr else {
        return None;
    };
    let Expr::FullCellPath(cell_path) = &lhs.expr else {
        return None;
    };
    let Expr::Var(var_id) = &cell_path.head.expr else {
        return None;
    };
    if cell_path.tail.is_empty() {
        return None;
    }

    let var_name = var_name_from_expr(&cell_path.head, context)?;
    let path = cell_path
        .tail
        .iter()
        .map(PathSegment::from_path_member)
        .collect();

    Some((
        *var_id,
        var_name,
        FieldAssignment {
            path,
            full_span: expr.span,
            value_span: rhs.span,
        },
    ))
}

fn pipeline_field_assignment(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<(VarId, String, FieldAssignment)> {
    (pipeline.elements.len() == 1)
        .then(|| extract_field_assignment(&pipeline.elements[0].expr, context))
        .flatten()
}

pub fn format_path(path: &[PathSegment]) -> String {
    path.iter()
        .map(|seg| match seg {
            PathSegment::Field(name) if !cell_path_member_needs_quotes(name) => name.clone(),
            PathSegment::Field(name) => format!("\"{name}\""),
            PathSegment::Index(idx) => idx.to_string(),
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// Tracks paths to detect conflicting assignments like `$x.a` and `$x.a.b`
#[derive(Default)]
struct ConflictTree {
    has_value: bool,
    children: Vec<(PathSegment, Self)>,
}

impl ConflictTree {
    fn can_insert(&self, path: &[PathSegment]) -> bool {
        match path.split_first() {
            None => !self.has_value && self.children.is_empty(),
            Some(_) if self.has_value => false,
            Some((segment, rest)) => self
                .children
                .iter()
                .find(|(s, _)| s == segment)
                .is_none_or(|(_, child)| child.can_insert(rest)),
        }
    }

    fn insert(&mut self, path: &[PathSegment]) {
        match path.split_first() {
            None => self.has_value = true,
            Some((segment, rest)) => {
                let child = self
                    .children
                    .iter_mut()
                    .find(|(s, _)| s == segment)
                    .map(|(_, c)| c);
                if let Some(c) = child {
                    c.insert(rest);
                } else {
                    let mut new_child = Self::default();
                    new_child.insert(rest);
                    self.children.push((segment.clone(), new_child));
                }
            }
        }
    }

    fn try_insert(&mut self, path: &[PathSegment]) -> bool {
        if self.can_insert(path) {
            self.insert(path);
            true
        } else {
            false
        }
    }
}

/// Builder for accumulating consecutive assignments to the same variable
struct GroupBuilder {
    root_var_id: VarId,
    root_var_name: String,
    assignments: Vec<FieldAssignment>,
    conflicts: ConflictTree,
}

impl GroupBuilder {
    fn new(var_id: VarId, var_name: String, first: FieldAssignment) -> Option<Self> {
        let mut conflicts = ConflictTree::default();
        conflicts.try_insert(&first.path).then_some(Self {
            root_var_id: var_id,
            root_var_name: var_name,
            assignments: vec![first],
            conflicts,
        })
    }

    fn try_extend(
        &mut self,
        var_id: VarId,
        assign: FieldAssignment,
    ) -> Result<(), FieldAssignment> {
        if var_id == self.root_var_id && self.conflicts.try_insert(&assign.path) {
            self.assignments.push(assign);
            Ok(())
        } else {
            Err(assign)
        }
    }

    fn into_group(self) -> Option<AssignmentGroup> {
        (self.assignments.len() >= 2).then(|| {
            let combined_span = Span::new(
                self.assignments.first().unwrap().full_span.start,
                self.assignments.last().unwrap().full_span.end,
            );
            let all_flat = self.assignments.iter().all(|a| a.path.len() == 1);
            AssignmentGroup {
                is_env: self.root_var_name == "env",
                root_var_name: self.root_var_name,
                assignments: self.assignments,
                combined_span,
                all_flat,
            }
        })
    }
}

enum FoldState {
    Empty,
    Building(GroupBuilder),
}

pub fn find_assignment_groups(block: &Block, context: &LintContext) -> Vec<AssignmentGroup> {
    let (groups, final_state) = block
        .pipelines
        .iter()
        .map(|p| pipeline_field_assignment(p, context))
        .fold(
            (Vec::new(), FoldState::Empty),
            |(mut groups, state), maybe_assign| match (state, maybe_assign) {
                (FoldState::Empty, Some((var_id, var_name, assign))) => {
                    match GroupBuilder::new(var_id, var_name, assign) {
                        Some(builder) => (groups, FoldState::Building(builder)),
                        None => (groups, FoldState::Empty),
                    }
                }
                (FoldState::Building(mut builder), Some((var_id, var_name, assign))) => {
                    match builder.try_extend(var_id, assign) {
                        Ok(()) => (groups, FoldState::Building(builder)),
                        Err(assign) => {
                            groups.extend(builder.into_group());
                            match GroupBuilder::new(var_id, var_name, assign) {
                                Some(new_builder) => (groups, FoldState::Building(new_builder)),
                                None => (groups, FoldState::Empty),
                            }
                        }
                    }
                }
                (FoldState::Building(builder), None) => {
                    groups.extend(builder.into_group());
                    (groups, FoldState::Empty)
                }
                (FoldState::Empty, None) => (groups, FoldState::Empty),
            },
        );

    match final_state {
        FoldState::Building(builder) => {
            let mut groups = groups;
            groups.extend(builder.into_group());
            groups
        }
        FoldState::Empty => groups,
    }
}

pub fn make_detection(group: &AssignmentGroup) -> Detection {
    let paths = group
        .assignments
        .iter()
        .map(|a| format_path(&a.path))
        .collect::<Vec<_>>()
        .join(", ");

    let base_detection = Detection::from_global_span(
        format!(
            "Consecutive ${} assignments can be merged: {paths}",
            group.root_var_name
        ),
        group.assignments[0].full_span,
    )
    .with_primary_label(format!(
        "{} consecutive ${} assignments",
        group.assignments.len(),
        group.root_var_name
    ));

    group
        .assignments
        .iter()
        .skip(1)
        .fold(base_detection, |det, assign| {
            det.with_extra_label(
                format!("${}.{}", group.root_var_name, format_path(&assign.path)),
                assign.full_span,
            )
        })
}
