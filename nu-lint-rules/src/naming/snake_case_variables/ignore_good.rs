use super::RULE;

#[test]
fn ignore_snake_case_let_variables() {
    let good_code = r#"
def good-func [] {
    let my_variable = 5
    let another_variable = 10
    let snake_case_name = "good"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_snake_case_mut_variables() {
    let good_code = "
def good-func [] {
    mut counter = 0
    mut total_sum = 100
    $counter += 1
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_single_letter_variables() {
    // Single lowercase letters are valid snake_case
    let good_code = "
def good-func [] {
    let x = 1
    let y = 2
    let z = 3
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_snake_case_with_numbers() {
    let good_code = r#"
def good-func [] {
    let var_1 = "first"
    let var_2 = "second"
    let item_count_3 = 100
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_snake_case_variables_with_type_annotation() {
    let good_code = "
let local: datetime = (date now)
mut count: int = 0
let items: list<string> = []
let point: record<x: int, y: int> = {x: 1, y: 2}
print $local $count $items $point
";

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_typed_variable_assigned_from_if_expression() {
    let good_code = r#"
export def "ps port" [port: int, --long(-l)]: nothing -> table {
	let pid: list<int> = if $nu.os-info.name == windows {
		netstat -ano | lines | where $it ends-with $":($port)"
	} else {
		error make 'ps port wrapper is only implemented for windows'
	}

	ps --long=$long | where pid in $pid
}
"#;

    RULE.assert_ignores(good_code);
}
