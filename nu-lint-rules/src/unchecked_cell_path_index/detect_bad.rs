use super::RULE;

#[test]
fn detect_cell_path_index_access() {
    RULE.assert_detects("$list.0");
}

#[test]
fn detect_nested_cell_path_index() {
    RULE.assert_detects("$data.items.0");
}

#[test]
fn detect_chained_access() {
    RULE.assert_detects("$coords.lat.0");
}

#[test]
fn detect_in_closure() {
    RULE.assert_detects(
        r#"
        $data | each {|row|
            $row.values.0
        }
    "#,
    );
}

#[test]
fn detect_in_function() {
    RULE.assert_detects(
        r#"
        def process-data [] {
            $in.results.0
        }
    "#,
    );
}

#[test]
fn detect_multiple_indices() {
    RULE.assert_detects("$matrix.0.1");
}
