#[cfg(test)]
mod tests {
    use crate::rules::avoid_jq_for_simple_ops::rule;

    #[test]
    fn ignore_nushell_length() {
        rule().assert_ignores("$data | length");
    }

    #[test]
    fn ignore_nushell_columns() {
        rule().assert_ignores("$record | columns");
    }

    #[test]
    fn ignore_nushell_describe() {
        rule().assert_ignores("$value | describe");
    }

    #[test]
    fn ignore_nushell_flatten() {
        rule().assert_ignores("$nested | flatten");
    }

    #[test]
    fn ignore_nushell_math_sum() {
        rule().assert_ignores("$numbers | math sum");
    }

    #[test]
    fn ignore_nushell_math_min() {
        rule().assert_ignores("$values | math min");
    }

    #[test]
    fn ignore_nushell_math_max() {
        rule().assert_ignores("$values | math max");
    }

    #[test]
    fn ignore_nushell_get_index() {
        rule().assert_ignores("$array | get 0");
    }

    #[test]
    fn ignore_complex_jq_operations() {
        rule().assert_ignores("^jq '.[] | select(.age > 30)' people.json");
    }

    #[test]
    fn ignore_jq_field_access() {
        rule().assert_ignores("^jq '.name' user.json");
    }

    #[test]
    fn ignore_other_external_commands() {
        rule().assert_ignores("^curl -s api.example.com");
    }

    #[test]
    fn ignore_builtin_commands() {
        rule().assert_ignores("open file.json | from json");
    }
}
