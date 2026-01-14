{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      localSystem:
      let
        pkgs = import nixpkgs {
          system = localSystem;
          overlays = [ (import rust-overlay) ];
        };

        # Rust toolchain for native builds
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common source filtering
        src = craneLib.cleanCargoSource ./.;

        # Common build arguments
        commonArgs = {
          inherit src;
          strictDeps = true;
          meta.mainProgram = "nu-lint";
        };

        # Build dependencies separately (cached by Nix/Cachix)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Native package for current system
        nativePackage = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

        # Cross-compilation helper
        mkCrossPackage =
          crossSystem:
          let
            crossPkgs = import nixpkgs {
              inherit crossSystem localSystem;
              overlays = [ (import rust-overlay) ];
            };

            # Rust target triple from crossSystem
            rustTarget =
              if crossSystem == "aarch64-linux" then
                "aarch64-unknown-linux-gnu"
              else
                throw "Unsupported cross target: ${crossSystem}";

            # Toolchain with cross-compilation target
            crossToolchain = pkgs.rust-bin.stable.latest.default.override {
              targets = [ rustTarget ];
            };

            crossCraneLib = (crane.mkLib crossPkgs).overrideToolchain crossToolchain;
            crossSrc = crossCraneLib.cleanCargoSource ./.;

            crossArgs = {
              src = crossSrc;
              strictDeps = true;
              meta.mainProgram = "nu-lint";
              CARGO_BUILD_TARGET = rustTarget;
              # Required by ring crate for cross-compiling assembly
              TARGET_CC = "${crossPkgs.stdenv.cc}/bin/${crossPkgs.stdenv.cc.targetPrefix}cc";
              HOST_CC = "${pkgs.stdenv.cc}/bin/cc";
            };

            crossCargoArtifacts = crossCraneLib.buildDepsOnly (
              crossArgs
              // {
                doCheck = false;
              }
            );
          in
          crossCraneLib.buildPackage (
            crossArgs
            // {
              cargoArtifacts = crossCargoArtifacts;
              doCheck = false; # Can't run cross-compiled tests
            }
          );
      in
      {
        packages = {
          default = nativePackage;
          x86_64-linux = nativePackage;
          aarch64-linux = mkCrossPackage "aarch64-linux";
          # Expose deps for caching
          deps = cargoArtifacts;
        };

        apps.default = {
          type = "app";
          program = pkgs.lib.getExe self.packages.${localSystem}.default;
        };

        devShells.default = craneLib.devShell {
          packages = [
            pkgs.rust-analyzer
          ];
        };
      }
    );
}
