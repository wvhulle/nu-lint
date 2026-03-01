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
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        src = pkgs.lib.cleanSource ./.;

        #
        # Toolchains
        #
        stableToolchain = pkgs.rust-bin.stable.latest.default;
        nightlyToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rustc-codegen-cranelift-preview" ];
        };

        stableCrane = (crane.mkLib pkgs).overrideToolchain stableToolchain;
        nightlyCrane = (crane.mkLib pkgs).overrideToolchain nightlyToolchain;

        #
        # Native builds
        #
        commonArgs = {
          inherit src;
          strictDeps = true;
          meta.mainProgram = "nu-lint";
        };

        llvmDeps = stableCrane.buildDepsOnly commonArgs;
        llvmPackage = stableCrane.buildPackage (commonArgs // { cargoArtifacts = llvmDeps; });

        craneliftDeps = nightlyCrane.buildDepsOnly (
          commonArgs // { RUSTFLAGS = "-Zcodegen-backend=cranelift"; }
        );
        craneliftPackage = nightlyCrane.buildPackage (
          commonArgs
          // {
            cargoArtifacts = craneliftDeps;
            RUSTFLAGS = "-Zcodegen-backend=cranelift";
          }
        );

        #
        # Static musl builds (portable Linux binaries)
        #
        mkMuslPackage =
          arch:
          let
            target = "${arch}-unknown-linux-musl";
            muslPkgs =
              if arch == "x86_64" then pkgs.pkgsCross.musl64 else pkgs.pkgsCross.aarch64-multiplatform-musl;

            toolchain = pkgs.rust-bin.stable.latest.default.override { targets = [ target ]; };
            muslCrane = (crane.mkLib pkgs).overrideToolchain (p: toolchain);

            cc = "${muslPkgs.stdenv.cc}/bin/${muslPkgs.stdenv.cc.targetPrefix}cc";
            ar = "${muslPkgs.stdenv.cc.bintools}/bin/${muslPkgs.stdenv.cc.targetPrefix}ar";
            targetEnv = builtins.replaceStrings [ "-" ] [ "_" ] target;

            args = {
              inherit src;
              strictDeps = true;
              pname = "nu-lint-static";
              meta.mainProgram = "nu-lint";
              doCheck = false;
              CARGO_BUILD_TARGET = target;
              "CARGO_TARGET_${pkgs.lib.toUpper targetEnv}_LINKER" = cc;
              "CC_${targetEnv}" = cc;
              "AR_${targetEnv}" = ar;
            };

            deps = muslCrane.buildDepsOnly args;
          in
          muslCrane.buildPackage (args // { cargoArtifacts = deps; });

        #
        # Pre-commit hooks
        #
        preCommitHooks = git-hooks.lib.${system}.run {
          inherit src;
          hooks.convco.enable = true;
        };

        #
        # Release targets (queried by build.nu)
        #
        releaseTargets =
          if pkgs.stdenv.isLinux then
            {
              "x86_64-unknown-linux-musl" = "x86_64-linux-musl";
              "aarch64-unknown-linux-musl" = "aarch64-linux-musl";
            }
          else if pkgs.stdenv.isDarwin then
            { "aarch64-apple-darwin" = "default"; }
          else
            { };
      in
      {
        inherit releaseTargets;

        packages = {
          default = llvmPackage;
          llvm = llvmPackage;
          cranelift = craneliftPackage;
          deps = llvmDeps;
          deps-cranelift = craneliftDeps;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          x86_64-linux-musl = mkMuslPackage "x86_64";
          aarch64-linux-musl = mkMuslPackage "aarch64";
        };

        checks = {
          inherit llvmPackage craneliftPackage;
          pre-commit = preCommitHooks;
        };

        apps = {
          default = {
            type = "app";
            program = pkgs.lib.getExe self.packages.${system}.default;
          };
          cranelift = {
            type = "app";
            program = pkgs.lib.getExe self.packages.${system}.cranelift;
          };
        };

        formatter =
          let
            inherit (preCommitHooks.config) package configFile;
          in
          pkgs.writeShellScriptBin "pre-commit-run" ''
            ${pkgs.lib.getExe package} run --all-files --config ${configFile}
          '';

        devShells = {
          default = stableCrane.devShell {
            inherit (preCommitHooks) shellHook;
            packages = [ pkgs.convco ];
          };
          cranelift = nightlyCrane.devShell {
            inherit (preCommitHooks) shellHook;
            packages = [ pkgs.convco ];
          };
        };
      }
    );
}
