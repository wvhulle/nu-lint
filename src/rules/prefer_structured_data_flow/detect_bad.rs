#[cfg(test)]
mod tests {
    use crate::rules::prefer_structured_data_flow::rule;

    #[test]
    fn detect_to_json_then_jq() {
        rule().assert_detects("$data | to json | ^jq '.field'");
    }

    #[test]
    fn detect_to_json_then_jq_complex() {
        rule().assert_detects("$records | to json | ^jq 'map(.name)'");
    }

    #[test]
    fn detect_to_json_then_jq_filter() {
        rule().assert_detects("$items | to json | ^jq 'select(.active)'");
    }

    #[test]
    fn detect_nested_to_json_jq() {
        rule().assert_detects("$data | select name != null | to json | ^jq '.[]'");
    }

    #[test]
    fn detect_to_json_pipe_jq_with_args() {
        rule().assert_detects("$config | to json | ^jq -r '.database.host'");
    }

    #[test]
    fn detect_to_json_multiline_jq() {
        rule().assert_detects(
            r#"$data | to json | ^jq '
            .users[]
            | select(.role == "admin")
            | .email
        '"#,
        );
    }
}
