use std::fs;

fn main() {
    let cargo_toml: toml::Table = fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml")
        .parse()
        .expect("Failed to parse Cargo.toml");

    let version = cargo_toml["dependencies"]["nu-parser"]
        .as_str()
        .expect("nu-parser version should be a string");

    println!("cargo:rustc-env=NU_PARSER_VERSION={version}");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
