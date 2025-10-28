#[cfg(test)]
mod tests {
    use crate::rules::prefer_structured_data_flow::rule;

    #[test]
    fn ignore_structured_data_operations() {
        rule().assert_ignores("$data | where field == 'value'");
    }

    #[test]
    fn ignore_structured_data_get() {
        rule().assert_ignores("$config | get database.host");
    }

    #[test]
    fn ignore_structured_data_each() {
        rule().assert_ignores("$users | each { get name }");
    }

    #[test]
    fn ignore_direct_jq_on_file() {
        rule().assert_ignores("^jq '.field' data.json");
    }

    #[test]
    fn ignore_to_json_for_output() {
        rule().assert_ignores("$data | to json | save output.json");
    }

    #[test]
    fn ignore_to_json_standalone() {
        rule().assert_ignores("$data | to json");
    }

    #[test]
    fn ignore_from_json() {
        rule().assert_ignores("$json_string | from json");
    }

    #[test]
    fn ignore_other_external_commands() {
        rule().assert_ignores("$data | to json | ^curl -d @- api.example.com");
    }

    #[test]
    fn ignore_separate_operations() {
        rule().assert_ignores(
            r"
            $data | to json | save temp.json
            ^jq '.field' temp.json
        ",
        );
    }

    #[test]
    fn ignore_structured_pipeline() {
        rule().assert_ignores("$data | where active | get name | sort");
    }
}
