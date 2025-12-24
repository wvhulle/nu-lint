use super::RULE;

#[test]
fn test_good_range_iteration() {
    let good = "0..10 | each { |i| print $i }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_loop_without_counter() {
    let good = "loop { if (check_done) { break } }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_loop_with_non_numeric_condition() {
    let good = "mut running = true; loop { if not $running { break }; $running = check_condition }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_while_loop() {
    let good = "mut x = 0; while $x < 10 { $x += 1 }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_for_loop() {
    let good = "for i in 0..10 { print $i }";
    RULE.assert_ignores(good);
}
