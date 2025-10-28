#[cfg(test)]
mod tests {
    use crate::rules::prefer_nushell_data_ops::rule;

    #[test]
    fn detect_jq_map() {
        rule().assert_detects("^jq 'map(.name)' users.json");
    }

    #[test]
    fn detect_jq_select() {
        rule().assert_detects("^jq 'select(.age > 30)' people.json");
    }

    #[test]
    fn detect_jq_group_by() {
        rule().assert_detects("^jq 'group_by(.category)' items.json");
    }

    #[test]
    fn detect_jq_array_iteration() {
        rule().assert_detects("^jq '.[]' data.json");
    }

    #[test]
    fn detect_jq_sort_by() {
        rule().assert_detects("^jq 'sort_by(.timestamp)' events.json");
    }

    #[test]
    fn detect_jq_unique() {
        rule().assert_detects("^jq 'unique' values.json");
    }

    #[test]
    fn detect_jq_reverse() {
        rule().assert_detects("^jq 'reverse' list.json");
    }

    #[test]
    fn detect_complex_jq_chain() {
        rule().assert_detects("^jq '.[] | select(.status == \"active\") | map(.id)' data.json");
    }
}
