use nu_protocol::{
    ast::{Argument, Block, Expr, Expression, ListItem, Pipeline, PipelineElement, RecordItem},
    engine::StateWorkingSet,
};

/// Print AST as JSON to stdout
pub fn print_ast(source: &str) {
    use crate::engine::{LintEngine, parse_source};

    let engine_state = LintEngine::new_state();
    let (block, working_set, _offset) = parse_source(engine_state, source.as_bytes());

    if !working_set.parse_errors.is_empty() {
        eprintln!("=== Parse Errors ===");
        for error in &working_set.parse_errors {
            eprintln!("{error:?}");
        }
        eprintln!();
    }

    let expanded = expand_block(&block, &working_set);
    let json = serde_json::to_string_pretty(&expanded).unwrap();
    println!("{json}");
}

/// Expand a block into a JSON value with all nested blocks expanded
pub fn expand_block(block: &Block, working_set: &StateWorkingSet) -> serde_json::Value {
    serde_json::json!({
        "pipelines": block.pipelines.iter().map(|p| expand_pipeline(p, working_set)).collect::<Vec<_>>(),
    })
}

/// Expand a pipeline into a JSON value
pub fn expand_pipeline(pipeline: &Pipeline, working_set: &StateWorkingSet) -> serde_json::Value {
    serde_json::json!({
        "elements": pipeline.elements.iter().map(|e| expand_pipeline_element(e, working_set)).collect::<Vec<_>>(),
    })
}

/// Expand a pipeline element into a JSON value
pub fn expand_pipeline_element(
    element: &PipelineElement,
    working_set: &StateWorkingSet,
) -> serde_json::Value {
    expand_expression(&element.expr, working_set)
}

/// Expand an expression into a JSON value, recursively expanding all nested structures
pub fn expand_expression(expr: &Expression, working_set: &StateWorkingSet) -> serde_json::Value {
    match &expr.expr {
        Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
            let block_type = match &expr.expr {
                Expr::Block(_) => "Block",
                Expr::Closure(_) => "Closure",
                Expr::Subexpression(_) => "Subexpression",
                _ => unreachable!(),
            };
            let nested_block = working_set.get_block(*block_id);
            serde_json::json!({
                "type": block_type,
                "block": expand_block(nested_block, working_set),
            })
        }
        Expr::Call(call) => {
            serde_json::json!({
                "type": "Call",
                "arguments": call.arguments.iter().map(|arg| {
                    match arg {
                        Argument::Positional(expr) => expand_expression(expr, working_set),
                        Argument::Named((name, short, expr)) => serde_json::json!({
                            "type": "Named",
                            "name": &name.item,
                            "short": short.as_ref().map(|s| &s.item),
                            "expr": expr.as_ref().map(|e| expand_expression(e, working_set)),
                        }),
                        Argument::Spread(expr) => serde_json::json!({
                            "type": "Spread",
                            "expr": expand_expression(expr, working_set),
                        }),
                        Argument::Unknown(expr) => serde_json::json!({
                            "type": "Unknown",
                            "expr": expand_expression(expr, working_set),
                        }),
                    }
                }).collect::<Vec<_>>(),
            })
        }
        Expr::Collect(_var_id, collect_expr) => {
            expand_expression(collect_expr, working_set)
        }
        Expr::String(s) => serde_json::Value::String(s.clone()),
        Expr::RawString(s) => serde_json::json!({
            "type": "RawString",
            "value": s,
        }),
        Expr::Int(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        Expr::Float(f) => serde_json::json!(f),
        Expr::Bool(b) => serde_json::Value::Bool(*b),
        Expr::Var(_var_id) => serde_json::json!({
            "type": "Var",
        }),
        Expr::FullCellPath(cell_path) => {
            if cell_path.tail.is_empty() {
                expand_expression(&cell_path.head, working_set)
            } else {
                serde_json::json!({
                    "type": "FullCellPath",
                    "head": expand_expression(&cell_path.head, working_set),
                    "tail": cell_path.tail.iter().map(|member| format!("{member:?}")).collect::<Vec<_>>(),
                })
            }
        }
        Expr::BinaryOp(lhs, op, rhs) => serde_json::json!({
            "type": "BinaryOp",
            "lhs": expand_expression(lhs, working_set),
            "op": expand_expression(op, working_set),
            "rhs": expand_expression(rhs, working_set),
        }),
        Expr::List(items) => {
            serde_json::json!({
                "type": "List",
                "items": items.iter().map(|item| {
                    match item {
                        ListItem::Item(expr) => expand_expression(expr, working_set),
                        ListItem::Spread(_span, expr) => serde_json::json!({
                            "type": "Spread",
                            "expr": expand_expression(expr, working_set),
                        }),
                    }
                }).collect::<Vec<_>>(),
            })
        }
        Expr::Record(items) => {
            serde_json::json!({
                "type": "Record",
                "items": items.iter().map(|item| {
                    match item {
                        RecordItem::Pair(key, value) => serde_json::json!({
                            "key": expand_expression(key, working_set),
                            "value": expand_expression(value, working_set),
                        }),
                        RecordItem::Spread(_span, expr) => serde_json::json!({
                            "type": "Spread",
                            "expr": expand_expression(expr, working_set),
                        }),
                    }
                }).collect::<Vec<_>>(),
            })
        }
        other => serde_json::json!({
            "type": format!("{other:?}").split('(').next().unwrap_or("Unknown"),
            "debug": format!("{other:?}"),
        }),
    }
}
