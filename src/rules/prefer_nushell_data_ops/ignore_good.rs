#[cfg(test)]
mod tests {
    use crate::rules::prefer_nushell_data_ops::rule;

    #[test]
    fn ignore_nushell_where() {
        rule().assert_ignores("$data | where age > 30");
    }

    #[test]
    fn ignore_nushell_each() {
        rule().assert_ignores("$items | each { get name }");
    }

    #[test]
    fn ignore_nushell_group_by() {
        rule().assert_ignores("$data | group-by category");
    }

    #[test]
    fn ignore_nushell_values() {
        rule().assert_ignores("$record | values");
    }

    #[test]
    fn ignore_nushell_sort_by() {
        rule().assert_ignores("$events | sort-by timestamp");
    }

    #[test]
    fn ignore_simple_jq_identity() {
        rule().assert_ignores("^jq '.' data.json");
    }

    #[test]
    fn ignore_simple_jq_field() {
        rule().assert_ignores("^jq '.name' user.json");
    }

    #[test]
    fn ignore_other_external_commands() {
        rule().assert_ignores("^grep pattern file.txt");
    }

    #[test]
    fn ignore_builtin_commands() {
        rule().assert_ignores("open file.json | from json");
    }
}
