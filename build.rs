use std::fs;

fn main() {
    let cargo_toml: toml::Table = fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml")
        .parse()
        .expect("Failed to parse Cargo.toml");

    let nu_parser_dep = &cargo_toml["dependencies"]["nu-parser"];

    let version = nu_parser_dep
        .as_str()
        .or_else(|| nu_parser_dep.get("version").and_then(|v| v.as_str()))
        .expect("nu-parser version not found in Cargo.toml");

    println!("cargo:rustc-env=NU_PARSER_VERSION={version}");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
