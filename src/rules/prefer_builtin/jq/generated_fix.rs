use super::rule;
use crate::context::LintContext;

#[test]
fn fix_jq_length() {
    let source = "^jq 'length' data.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        let violation = &violations[0];

        if let Some(fix) = &violation.fix {
            assert!(
                fix.replacements[0]
                    .new_text
                    .contains("open $file | from json | length")
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_keys() {
    let source = "^jq 'keys' object.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("columns"),
                "Should suggest 'columns' for jq 'keys'"
            );
            assert!(
                new_text.contains("from json"),
                "Should include 'from json' for file input"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_to_json_then_jq_add() {
    let source = "$numbers | to json | ^jq 'add'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("math sum"),
                "Should suggest 'math sum' for jq 'add'"
            );
            assert!(
                !new_text.contains("from json"),
                "Should not include 'from json' when input is piped data"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_to_json_then_jq_length() {
    let source = "$values | to json | ^jq 'length'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("length"),
                "Should suggest 'length' for jq 'length'"
            );
            assert_eq!(
                new_text, "length",
                "For piped data, should just be 'length'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_to_json_then_jq_sort() {
    let source = "$items | to json | ^jq 'sort'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("sort"),
                "Should suggest 'sort' for jq 'sort'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_to_json_then_jq_unique() {
    let source = "$data | to json | ^jq 'unique'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("uniq"),
                "Should suggest 'uniq' for jq 'unique'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_to_json_then_jq_flatten() {
    let source = "$nested | to json | ^jq 'flatten'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("flatten"),
                "Should suggest 'flatten' for jq 'flatten'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_array_index() {
    let source = "^jq '.[0]' data.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("get 0"),
                "Should suggest 'get 0' for jq '.[0]'"
            );
            assert!(
                new_text.contains("from json"),
                "Should include 'from json' for file input"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_negative_index() {
    let source = "^jq '.[-1]' items.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("last"),
                "Should suggest 'last' for jq '.[-1]'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_field_access() {
    let source = "^jq '.name' user.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("get name"),
                "Should suggest 'get name' for jq '.name'"
            );
            assert!(
                new_text.contains("from json"),
                "Should include 'from json' for file input"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_nested_field_access() {
    let source = "^jq '.user.email' data.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("get user.email"),
                "Should suggest 'get user.email' for jq '.user.email'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_array_iteration() {
    let source = "^jq '.[]' array.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("each"),
                "Should suggest 'each' for jq '.[]'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_field_array_iteration() {
    let source = "^jq '.users[]' data.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("get users | each"),
                "Should suggest 'get users | each' for jq '.users[]'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_map_field() {
    let source = "$users | to json | ^jq 'map(.name)'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("get name"),
                "Should suggest 'get name' for jq 'map(.name)'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_group_by() {
    let source = "$records | to json | ^jq 'group_by(.category)'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("group-by category"),
                "Should suggest 'group-by category' for jq 'group_by(.category)'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_sort_by() {
    let source = "$events | to json | ^jq 'sort_by(.timestamp)'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("sort-by timestamp"),
                "Should suggest 'sort-by timestamp' for jq 'sort_by(.timestamp)'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_min() {
    let source = "$numbers | to json | ^jq 'min'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("math min"),
                "Should suggest 'math min' for jq 'min'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_max() {
    let source = "$numbers | to json | ^jq 'max'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("math max"),
                "Should suggest 'math max' for jq 'max'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_type() {
    let source = "^jq 'type' value.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("describe"),
                "Should suggest 'describe' for jq 'type'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_jq_reverse() {
    let source = "$list | to json | ^jq 'reverse'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("reverse"),
                "Should suggest 'reverse' for jq 'reverse'"
            );
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_message_explains_performance_benefit() {
    let source = "^jq 'length' data.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1);

        let violation = &violations[0];
        assert!(
            violation.message.contains("Nushell") || violation.message.contains("built-in"),
            "Message should mention Nushell or built-ins"
        );
    });
}

#[test]
fn fix_preserves_file_argument() {
    let source = "^jq '.name' /path/to/user.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                new_text.contains("open $file"),
                "Should preserve file argument with 'open $file'"
            );
            assert!(new_text.contains("get name"), "Should convert field access");
        } else {
            panic!("Expected fix to be present");
        }
    });
}

#[test]
fn fix_handles_piped_data_without_open() {
    let source = "$data | to json | ^jq '.name'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 1, "Should detect one violation");

        if let Some(fix) = &violations[0].fix {
            let new_text = &fix.replacements[0].new_text;
            assert!(
                !new_text.contains("open"),
                "Should not use 'open' for piped data"
            );
            assert!(new_text.contains("get name"), "Should convert field access");
        } else {
            panic!("Expected fix to be present");
        }
    });
}
