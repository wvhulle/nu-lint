use super::rule;

#[test]
fn test_good_let_variables() {
    let good_code = r#"
def good-func [] {
    let my_variable = 5
    let another_variable = 10
    let snake_case_name = "good"
}
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn test_good_mut_variables() {
    let good_code = "
def good-func [] {
    mut counter = 0
    mut total_sum = 100
    $counter += 1
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_good_single_letter_variables() {
    // Single lowercase letters are valid snake_case
    let good_code = "
def good-func [] {
    let x = 1
    let y = 2
    let z = 3
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_good_variables_with_numbers() {
    let good_code = r#"
def good-func [] {
    let var_1 = "first"
    let var_2 = "second"
    let item_count_3 = 100
}
"#;

    rule().assert_ignores(good_code);
}
