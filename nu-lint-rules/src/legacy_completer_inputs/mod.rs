use std::collections::BTreeMap;

use nu_protocol::{
    CommandWideCompleter, Completion, DeclId, ENV_VARIABLE_ID, PositionalArg, Signature, Span,
    VarId,
    ast::{Assignment, Expr, Expression, Operator, PathMember, RecordItem, Traverse},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, external::literal_content},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

const COMPLETER_INPUTS: [&str; 3] = ["token", "place", "buffer"];

const EXTERNAL_COMPLETER_PATH: [&str; 4] = ["config", "completions", "external", "completer"];

#[derive(Clone, Copy)]
enum CompleterKind {
    Parameter,
    CommandWide,
    External,
}

impl CompleterKind {
    const fn usage(self) -> &'static str {
        match self {
            Self::Parameter => "completes a parameter",
            Self::CommandWide => "completes a whole command",
            Self::External => "completes external commands",
        }
    }

    const fn replacement(self, position: usize) -> &'static str {
        match (self, position) {
            (Self::Parameter, 0) => "rename to `buffer: string`, the line up to the cursor",
            (Self::Parameter, _) => "rename to `place: record` and read `$place.cursor`",
            (Self::CommandWide | Self::External, 0) => {
                "rename to `place: record` and read `$place.command`"
            }
            (Self::CommandWide | Self::External, _) => "always empty, remove it",
        }
    }
}

struct LegacyInput<'a> {
    position: usize,
    input: &'a PositionalArg,
}

fn legacy_inputs(signature: &Signature) -> Vec<LegacyInput<'_>> {
    signature
        .required_positional
        .iter()
        .chain(&signature.optional_positional)
        .take(2)
        .enumerate()
        .filter(|(_, input)| !COMPLETER_INPUTS.contains(&input.name.as_str()))
        .map(|(position, input)| LegacyInput { position, input })
        .collect()
}

fn legacy_detection(
    subject: &str,
    signature: &Signature,
    kind: CompleterKind,
    span: Span,
    context: &LintContext,
) -> Option<Detection> {
    let inputs = legacy_inputs(signature);
    let quoted_names: Vec<_> = inputs
        .iter()
        .map(|legacy| format!("`{}`", legacy.input.name))
        .collect();
    let message = match quoted_names.as_slice() {
        [] => return None,
        [name] => format!("{subject} uses the deprecated positional input {name}"),
        names => format!(
            "{subject} uses the deprecated positional inputs {}",
            names.join(" and ")
        ),
    };
    let detection = Detection::from_global_span(message, span).with_primary_label(kind.usage());
    Some(inputs.iter().fold(detection, |detection, legacy| {
        let Some(var_id) = legacy.input.var_id else {
            return detection;
        };
        detection.with_extra_label(
            kind.replacement(legacy.position),
            context.working_set.get_variable(var_id).declaration_span,
        )
    }))
}

fn completion_command(completion: Option<&Completion>) -> Option<DeclId> {
    match completion? {
        Completion::Command(decl_id) => Some(*decl_id),
        Completion::List(_) | Completion::Builtin(_) => None,
    }
}

struct CompleterReference {
    completer: DeclId,
    kind: CompleterKind,
}

fn referenced_completers(signature: &Signature) -> Vec<CompleterReference> {
    let positional_completers = signature
        .required_positional
        .iter()
        .chain(&signature.optional_positional)
        .chain(&signature.rest_positional)
        .filter_map(|positional| completion_command(positional.completion.as_ref()));
    let flag_completers = signature
        .named
        .iter()
        .filter_map(|flag| completion_command(flag.completion.as_ref()));
    let command_wide_completer = match signature.complete {
        Some(CommandWideCompleter::Command(completer)) => Some(CompleterReference {
            completer,
            kind: CompleterKind::CommandWide,
        }),
        Some(CommandWideCompleter::External) | None => None,
    };
    positional_completers
        .chain(flag_completers)
        .map(|completer| CompleterReference {
            completer,
            kind: CompleterKind::Parameter,
        })
        .chain(command_wide_completer)
        .collect()
}

fn file_completers(context: &LintContext) -> BTreeMap<DeclId, CompleterKind> {
    let (first_file_decl, decl_count) = context.new_decl_range();
    let mut completers = BTreeMap::new();
    (first_file_decl..decl_count)
        .map(DeclId::new)
        .flat_map(|decl_id| {
            referenced_completers(&context.working_set.get_decl(decl_id).signature())
        })
        .filter(|reference| reference.completer.get() >= first_file_decl)
        .for_each(|reference| {
            completers
                .entry(reference.completer)
                .or_insert(reference.kind);
        });
    completers
}

fn command_completer_detections(context: &LintContext) -> Vec<Detection> {
    let definitions = context.custom_commands();
    file_completers(context)
        .into_iter()
        .filter_map(|(completer, kind)| {
            let body = context.working_set.get_decl(completer).block_id()?;
            let definition = definitions.iter().find(|def| def.body == body)?;
            legacy_detection(
                &format!("Completer `{}`", definition.name),
                &definition.signature,
                kind,
                definition.name_span,
                context,
            )
        })
        .collect()
}

fn env_cell_path_keys(expr: &Expression) -> Option<Vec<&str>> {
    let Expr::FullCellPath(cell_path) = &expr.expr else {
        return None;
    };
    if !matches!(cell_path.head.expr, Expr::Var(ENV_VARIABLE_ID)) {
        return None;
    }
    cell_path
        .tail
        .iter()
        .map(|member| match member {
            PathMember::String { val, .. } => Some(val.as_str()),
            PathMember::Int { .. } => None,
        })
        .collect()
}

fn unwrap_value<'a>(expr: &'a Expression, context: &'a LintContext) -> &'a Expression {
    match &expr.expr {
        Expr::Block(block_id) | Expr::Subexpression(block_id) => {
            match context
                .working_set
                .get_block(*block_id)
                .pipelines
                .as_slice()
            {
                [pipeline] => match pipeline.elements.as_slice() {
                    [element] => unwrap_value(&element.expr, context),
                    _ => expr,
                },
                _ => expr,
            }
        }
        Expr::FullCellPath(cell_path) if cell_path.tail.is_empty() => {
            unwrap_value(&cell_path.head, context)
        }
        _ => expr,
    }
}

fn record_value<'a>(
    record: &'a Expression,
    key: &str,
    context: &'a LintContext,
) -> Option<&'a Expression> {
    let Expr::Record(items) = &unwrap_value(record, context).expr else {
        return None;
    };
    items.iter().find_map(|item| match item {
        RecordItem::Pair(name, value) if literal_content(name) == Some(key) => Some(value),
        _ => None,
    })
}

fn assigned_external_completer<'a>(
    expr: &'a Expression,
    context: &'a LintContext,
) -> Option<&'a Expression> {
    let Expr::BinaryOp(target, operator, value) = &expr.expr else {
        return None;
    };
    if !matches!(
        operator.expr,
        Expr::Operator(Operator::Assignment(Assignment::Assign))
    ) {
        return None;
    }
    let assigned_keys = env_cell_path_keys(target)?;
    let nested_keys = EXTERNAL_COMPLETER_PATH.strip_prefix(assigned_keys.as_slice())?;
    nested_keys.iter().try_fold(value.as_ref(), |record, key| {
        record_value(record, key, context)
    })
}

fn variable_initializer<'a>(var_id: VarId, context: &'a LintContext) -> Option<&'a Expression> {
    let mut initializers = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| match &expr.expr {
            Expr::Call(call)
                if call
                    .extract_variable_declaration(context)
                    .is_some_and(|(declared, _, _)| declared == var_id) =>
            {
                call.get_positional_arg(1).into_iter().collect()
            }
            _ => vec![],
        },
        &mut initializers,
    );
    initializers.into_iter().next()
}

fn closure_literal<'a>(expr: &'a Expression, context: &'a LintContext) -> Option<&'a Expression> {
    let value = unwrap_value(expr, context);
    match &value.expr {
        Expr::Closure(_) => Some(value),
        Expr::Var(var_id) => closure_literal(variable_initializer(*var_id, context)?, context),
        _ => None,
    }
}

fn external_completer_detections(context: &LintContext) -> Vec<Detection> {
    context.detect_single(|expr, ctx| {
        let completer = assigned_external_completer(expr, ctx)?;
        let closure = closure_literal(completer, ctx)?;
        let Expr::Closure(block_id) = closure.expr else {
            return None;
        };
        let signature = &ctx.working_set.get_block(block_id).signature;
        legacy_detection(
            "External completer closure",
            signature,
            CompleterKind::External,
            closure.span,
            ctx,
        )
    })
}

struct LegacyCompleterInputs;

impl DetectFix for LegacyCompleterInputs {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "legacy_completer_inputs"
    }

    fn short_description(&self) -> &'static str {
        "Custom completer uses deprecated positional inputs"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Since Nushell 0.116.0, custom completers receive one record whose fields bind to \
             parameters named `token`, `place` and `buffer`. Completers whose first parameters \
             have other names, such as `[context, offset]` or `{|spans| ...}`, still run through \
             a compatibility bridge that will be removed in a future release.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_completions.html#context-aware-custom-completions")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(
            command_completer_detections(context)
                .into_iter()
                .chain(external_completer_detections(context))
                .collect(),
        )
    }
}

pub static RULE: &dyn Rule = &LegacyCompleterInputs;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
