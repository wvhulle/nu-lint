use super::RULE;

#[test]
fn detect_cat_of_one_file() {
    RULE.assert_detects("^cat file.txt");
    RULE.assert_detects("^cat config.json | ^jq .name");
    RULE.assert_detects("def show [path] { ^cat $path }");
}
