#[cfg(test)]
mod tests {
    use crate::rules::prefer_from_json::rule;

    #[test]
    fn detect_simple_jq_identity() {
        rule().assert_detects("^jq '.' data.json");
    }

    #[test]
    fn detect_jq_field_access() {
        rule().assert_detects("^jq '.name' users.json");
    }

    #[test]
    fn detect_jq_nested_field() {
        rule().assert_detects("^jq '.user.email' config.json");
    }

    #[test]
    fn detect_jq_without_file() {
        rule().assert_detects("cat data.json | ^jq '.'");
    }

    #[test]
    fn detect_jq_complex_filter() {
        rule().assert_detects("^jq '.[] | select(.age > 30)' people.json");
    }

    #[test]
    fn detect_jq_array_access() {
        rule().assert_detects("^jq '.[0]' items.json");
    }

    #[test]
    fn detect_jq_keys() {
        rule().assert_detects("^jq 'keys' object.json");
    }

    #[test]
    fn detect_jq_length() {
        rule().assert_detects("^jq 'length' array.json");
    }
}
