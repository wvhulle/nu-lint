use super::rule;
#[test]
fn test_good_parse_command() {
    let good = "'name:john age:30' | parse '{name}:{age}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_with_patterns() {
    let good = "'User: alice, ID: 123' | parse 'User: {name}, ID: {id}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_simple_split() {
    let good = "'a,b,c' | split row ','";
    rule().assert_ignores(good);
}

#[test]
fn test_good_split_for_iteration() {
    let good = "'a,b,c' | split row ',' | each { |item| $item | str upcase }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_split_column() {
    let good = "'name,age,city' | split column ',' name age city";
    rule().assert_ignores(good);
}

#[test]
fn test_good_from_csv() {
    let good = "'name,age\njohn,30\njane,25' | from csv";
    rule().assert_ignores(good);
}
