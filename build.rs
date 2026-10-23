use std::fs;

use toml::{Table, Value};

fn dependency_version(dependency: &Value) -> Option<&str> {
    dependency
        .as_str()
        .or_else(|| dependency.get("version").and_then(Value::as_str))
}

fn nu_parser_version(cargo_toml: &Table) -> Option<&str> {
    let package_dependency = cargo_toml.get("dependencies")?.get("nu-parser")?;
    dependency_version(package_dependency).or_else(|| {
        let workspace_dependency = cargo_toml
            .get("workspace")?
            .get("dependencies")?
            .get("nu-parser")?;
        dependency_version(workspace_dependency)
    })
}

fn main() {
    let cargo_toml: Table = fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml")
        .parse()
        .expect("Failed to parse Cargo.toml");

    let version =
        nu_parser_version(&cargo_toml).expect("nu-parser version not found in Cargo.toml");

    println!("cargo:rustc-env=NU_PARSER_VERSION={version}");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
