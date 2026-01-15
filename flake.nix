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

        # Stable toolchain for LLVM backend (default)
        rustToolchainLLVM = pkgs.rust-bin.stable.latest.default;
        craneLibLLVM = (crane.mkLib pkgs).overrideToolchain rustToolchainLLVM;

        # Nightly toolchain with Cranelift backend component
        rustToolchainCranelift = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rustc-codegen-cranelift-preview" ];
        };
        craneLibCranelift = (crane.mkLib pkgs).overrideToolchain rustToolchainCranelift;

        src = craneLibLLVM.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          meta.mainProgram = "nu-lint";
        };

        # LLVM backend (default, optimized)
        cargoArtifactsLLVM = craneLibLLVM.buildDepsOnly commonArgs;
        nativePackageLLVM = craneLibLLVM.buildPackage (
          commonArgs
          // {
            cargoArtifacts = cargoArtifactsLLVM;
          }
        );

        # Cranelift backend (faster compilation, slower runtime)
        craneliftArgs = commonArgs // {
          RUSTFLAGS = "-Zcodegen-backend=cranelift";
        };
        cargoArtifactsCranelift = craneLibCranelift.buildDepsOnly craneliftArgs;
        nativePackageCranelift = craneLibCranelift.buildPackage (
          craneliftArgs
          // {
            cargoArtifacts = cargoArtifactsCranelift;
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
          default = nativePackageLLVM;
          llvm = nativePackageLLVM;
          cranelift = nativePackageCranelift;
          x86_64-linux = nativePackageLLVM;
          aarch64-linux = mkCrossPackage "aarch64-linux";
          deps = cargoArtifactsLLVM;
          deps-cranelift = cargoArtifactsCranelift;
        };

        checks = {
          inherit nativePackageLLVM nativePackageCranelift;
          pre-commit = preCommitHooks;
        };

        apps = {
          default = {
            type = "app";
            program = pkgs.lib.getExe self.packages.${localSystem}.default;
          };
          cranelift = {
            type = "app";
            program = pkgs.lib.getExe self.packages.${localSystem}.cranelift;
          };
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
          # LLVM toolchain dev shell
          llvm = craneLibLLVM.devShell {
            checks = self.checks;
            inherit (preCommitHooks) shellHook;
            packages = [
              pkgs.convco
            ];
          };
          # Cranelift toolchain dev shell (for faster dev builds)
          cranelift = craneLibCranelift.devShell {
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
