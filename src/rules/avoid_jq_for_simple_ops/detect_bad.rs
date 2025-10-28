#[cfg(test)]
mod tests {
    use crate::rules::avoid_jq_for_simple_ops::rule;

    #[test]
    fn detect_jq_length() {
        rule().assert_detects("^jq 'length' data.json");
    }

    #[test]
    fn detect_jq_keys() {
        rule().assert_detects("^jq 'keys' object.json");
    }

    #[test]
    fn detect_jq_type() {
        rule().assert_detects("^jq 'type' value.json");
    }

    #[test]
    fn detect_jq_empty() {
        rule().assert_detects("^jq 'empty' file.json");
    }

    #[test]
    fn detect_jq_not() {
        rule().assert_detects("^jq 'not' boolean.json");
    }

    #[test]
    fn detect_jq_flatten() {
        rule().assert_detects("^jq 'flatten' nested.json");
    }

    #[test]
    fn detect_jq_add() {
        rule().assert_detects("^jq 'add' numbers.json");
    }

    #[test]
    fn detect_jq_min() {
        rule().assert_detects("^jq 'min' values.json");
    }

    #[test]
    fn detect_jq_max() {
        rule().assert_detects("^jq 'max' values.json");
    }

    #[test]
    fn detect_jq_array_index() {
        rule().assert_detects("^jq '.[0]' array.json");
    }

    #[test]
    fn detect_jq_array_index_negative() {
        rule().assert_detects("^jq '.[-1]' array.json");
    }
}
