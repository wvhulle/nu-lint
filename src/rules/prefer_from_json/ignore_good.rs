#[cfg(test)]
mod tests {
    use crate::rules::prefer_from_json::rule;

    #[test]
    fn ignore_from_json() {
        rule().assert_ignores("open data.json | from json");
    }

    #[test]
    fn ignore_structured_data_access() {
        rule().assert_ignores("open users.json | from json | get name");
    }

    #[test]
    fn ignore_where_filter() {
        rule().assert_ignores("open people.json | from json | where age > 30");
    }

    #[test]
    fn ignore_native_json_parsing() {
        rule().assert_ignores("$data | from json | get field");
    }

    #[test]
    fn ignore_json_conversion() {
        rule().assert_ignores("$record | to json");
    }

    #[test]
    fn ignore_other_external_commands() {
        rule().assert_ignores("^curl -s api.example.com");
    }

    #[test]
    fn ignore_builtin_commands() {
        rule().assert_ignores("open file.txt | lines");
    }

    #[test]
    fn ignore_jq_as_variable() {
        rule().assert_ignores("let jq_result = $data | from json");
    }
}
