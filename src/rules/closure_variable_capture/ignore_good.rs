use super::rule;

#[test]
fn ignore_proper_in_usage() {
    let good_code = r"
let items = [1, 2, 3]
$items | each { $in * 10 }";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_builtin_variables() {
    let good_code = r"
$env.PATH | each { $in | path exists }";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_proper_where_usage() {
    let good_code = r"
$data | where { $in.value > 5 }";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_it_variable() {
    let good_code = r"
$items | each { $it.name }";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_do_block_with_local_variable() {
    let good_code = r#"
def run_validation [temp_config: string, is_nixos: bool] {
  let cmd = if $is_nixos {
    ["nixos-rebuild" "dry-build" "-I" $"nixos-config=($temp_config)"]
  } else {
    ["nix-instantiate" "--eval" "--strict" $temp_config]
  }

  do { ^$cmd.0 ...($cmd | skip 1) } | complete
}"#;
    rule().assert_ignores(good_code);
}
