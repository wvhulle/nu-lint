use super::RULE;

#[test]
fn fix_plain_read() {
    RULE.assert_fixed_is("^read name", "let name = input");
}

#[test]
fn fix_prompt() {
    RULE.assert_fixed_is("^read -r -p 'Name: ' name", "let name = input 'Name: '");
}

#[test]
fn fix_combined_flags_with_prompt() {
    RULE.assert_fixed_is(
        r#"^read -rsp "Password: " password"#,
        r#"let password = input --suppress-output "Password: ""#,
    );
}

#[test]
fn fix_attached_prompt() {
    RULE.assert_fixed_is("^read -pName: name", "let name = input 'Name:'");
}
