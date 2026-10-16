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

        devTools = [
          pkgs.prek
          pkgs.convco
          pkgs.release-plz
          pkgs.git-cliff
        ];

        mkApp = drv: {
          type = "app";
          program = pkgs.lib.getExe drv;
        };

        releaseScript = pkgs.writeShellApplication {
          name = "release";
          runtimeInputs = [
            nightlyToolchain
            pkgs.release-plz
            pkgs.gitMinimal
          ];
          text = ''
            release-plz release --forge gitea --git-token "$CODEBERG_TOKEN"
            release-plz release-pr --forge gitea --git-token "$CODEBERG_TOKEN"
          '';
        };

        checkCommitsScript = pkgs.writeShellApplication {
          name = "check-commits";
          runtimeInputs = [
            pkgs.convco
            pkgs.gitMinimal
          ];
          text = ''
            convco check "$@"
          '';
        };

        muslPackages = {
          x86_64 = mkMuslPackage "x86_64";
          aarch64 = mkMuslPackage "aarch64";
        };

        releaseArchives = pkgs.runCommand "nu-lint-dist" { } (
          ''
            mkdir -p $out
          ''
          + pkgs.lib.concatMapAttrsStringSep "\n" (arch: package: ''
            tar -czf $out/nu-lint-${arch}-unknown-linux-musl.tar.gz -C ${package}/bin nu-lint
          '') muslPackages
        );
      in
      {
        packages = {
          default = llvmPackage;
          llvm = llvmPackage;
          cranelift = craneliftPackage;
          deps = llvmDeps;
          deps-cranelift = craneliftDeps;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          x86_64-linux-musl = muslPackages.x86_64;
          aarch64-linux-musl = muslPackages.aarch64;
          dist = releaseArchives;
        };

        checks = {
          tests = llvmPackage;
          fmt = nightlyCrane.cargoFmt { inherit src; };
        };

        apps = {
          default = mkApp self.packages.${system}.default;
          cranelift = mkApp self.packages.${system}.cranelift;
          release = mkApp releaseScript;
          check-commits = mkApp checkCommitsScript;
        };

        devShells.default = nightlyCrane.devShell {
          packages = devTools;
        };
      }
    );
}
