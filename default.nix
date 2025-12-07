{
  pkgs ? import <nixpkgs> { },
  fetchFromGitHub ? pkgs.fetchFromGitHub,
}:
let
  meta = builtins.fromTOML (pkgs.lib.readFile ./Cargo.toml);

  fenix = pkgs.callPackage (fetchFromGitHub {
    owner = "nix-community";
    repo = "fenix";
    rev = "b0fa429fc946e6e716dff3bfb97ce6383eae9359";
    hash = "sha256-YmnUYXjacFHa8fWCo8gBAHpqlcG8+P5+5YYFhy6hOkg=";
  }) { };

  toolchain = fenix.fromToolchainFile {
    file = ./rust-toolchain.toml;
    sha256 = "sha256-IQUcjhizZsNE1NYkdrwkVNxGpUlujMlfy8tdcbp7NnQ=";
  };

  rustPlatform = pkgs.makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
rustPlatform.buildRustPackage {
  pname = meta.package.name;
  version = meta.package.version;

  src = pkgs.lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  doCheck = false;

  nativeBuildInputs = [ ];

  buildInputs = [ ];

  meta = with pkgs.lib; {
    description = meta.package.description;
    homepage = meta.package.repository;
    license = licenses.mit;
    maintainers = meta.package.authors;
    mainProgram = meta.package.name;
    platforms = platforms.all;
  };
}
