{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
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
      git-hooks,
    }:
    flake-utils.lib.eachDefaultSystem (
      localSystem:
      let
        pkgs = import nixpkgs {
          system = localSystem;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          meta.mainProgram = "nu-lint";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        nativePackage = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

        mkCrossPackage =
          crossSystem:
          let
            crossPkgs = import nixpkgs {
              inherit crossSystem localSystem;
              overlays = [ (import rust-overlay) ];
            };

            rustTarget =
              if crossSystem == "aarch64-linux" then
                "aarch64-unknown-linux-gnu"
              else
                throw "Unsupported cross target: ${crossSystem}";

            crossToolchain = pkgs.rust-bin.stable.latest.default.override {
              targets = [ rustTarget ];
            };

            crossCraneLib = (crane.mkLib crossPkgs).overrideToolchain (p: crossToolchain);
            crossSrc = crossCraneLib.cleanCargoSource ./.;

            crossArgs = {
              src = crossSrc;
              strictDeps = true;
              meta.mainProgram = "nu-lint";
              CARGO_BUILD_TARGET = rustTarget;
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
              doCheck = false;
            }
          );

        preCommitHooks = git-hooks.lib.${localSystem}.run {
          src = ./.;
          hooks = {
            nixfmt.enable = true;
            convco.enable = true;
          };
        };
      in
      {
        # Run pre-commit hooks with `nix fmt`
        formatter =
          let
            config = preCommitHooks.config;
            inherit (config) package configFile;
          in
          pkgs.writeShellScriptBin "pre-commit-run" ''
            ${pkgs.lib.getExe package} run --all-files --config ${configFile}
          '';

        packages = {
          default = nativePackage;
          x86_64-linux = nativePackage;
          aarch64-linux = mkCrossPackage "aarch64-linux";
          deps = cargoArtifacts;
        };

        checks = {
          inherit nativePackage;
          pre-commit = preCommitHooks;
        };

        apps.default = {
          type = "app";
          program = pkgs.lib.getExe self.packages.${localSystem}.default;
        };

        devShells = {
          # I prefer to use the toolchain provided by my ~/.rustup installation for ease of use with the cargo commands
          default = pkgs.mkShell {
            inherit (preCommitHooks) shellHook;
            packages = [
              pkgs.rustup
              pkgs.convco
            ];
          };
          # In case you want the exact same toolchain as the build
          crane = craneLib.devShell {
            checks = self.checks;
            inherit (preCommitHooks) shellHook;
            packages = [
              pkgs.convco
            ];
          };
        };
      }
    );
}
