{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nu-scripts = {
      url = "github:nushell/nu_scripts";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      nu-scripts,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        src = pkgs.lib.cleanSourceWith {
          src = pkgs.lib.cleanSource ./.;
          filter = path: _: baseNameOf path != "demos";
        };

        #
        # Toolchains
        #
        stableToolchain = pkgs.rust-bin.stable.latest.default;
        ciToolchain = pkgs.rust-bin.stable.latest.minimal;
        nightlyToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rustc-codegen-cranelift-preview" ];
        };

        rustfmtToolchain = pkgs.rust-bin.selectLatestNightlyWith (
          toolchain: toolchain.minimal.override { extensions = [ "rustfmt" ]; }
        );

        stableCrane = (crane.mkLib pkgs).overrideToolchain stableToolchain;
        ciCrane = (crane.mkLib pkgs).overrideToolchain ciToolchain;
        nightlyCrane = (crane.mkLib pkgs).overrideToolchain nightlyToolchain;
        rustfmtCrane = (crane.mkLib pkgs).overrideToolchain rustfmtToolchain;

        #
        # Native builds
        #
        commonArgs = {
          inherit src;
          strictDeps = true;
          meta.mainProgram = "nu-lint";
        };

        llvmDeps = stableCrane.buildDepsOnly commonArgs;
        llvmPackage = stableCrane.buildPackage (
          commonArgs
          // {
            cargoArtifacts = llvmDeps;
            doCheck = false;
          }
        );

        xtaskPackage = stableCrane.buildPackage (
          commonArgs
          // {
            pname = "xtask";
            cargoArtifacts = llvmDeps;
            cargoExtraArgs = "--locked --package xtask";
            doCheck = false;
            meta.mainProgram = "xtask";
          }
        );

        recordings = import ./demos {
          inherit pkgs;
          nu-lint = llvmPackage;
          xtask = xtaskPackage;
        };

        devProfileArgs = commonArgs // {
          CARGO_PROFILE = "dev";
          postUnpack = ''
            if [[ "$NIX_BUILD_TOP" != /build ]]; then
              rm -rf /tmp/nu-lint-source
              mv "$sourceRoot" /tmp/nu-lint-source
              sourceRoot=/tmp/nu-lint-source
            fi
          '';
        };
        devProfileDeps = ciCrane.buildDepsOnly devProfileArgs;
        tests = ciCrane.cargoTest (
          devProfileArgs
          // {
            cargoArtifacts = devProfileDeps;
            doInstallCargoArtifacts = false;
            cargoTestExtraArgs = "--workspace";
          }
        );

        astGrepCheck = pkgs.runCommand "nu-lint-ast-grep" { nativeBuildInputs = [ pkgs.ast-grep ]; } ''
          cd ${src}
          ast-grep test --skip-snapshot-tests
          ast-grep scan
          touch $out
        '';

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
          pkgs.ast-grep
          pkgs.cargo-edit
          pkgs.prek
          pkgs.convco
          pkgs.release-plz
          pkgs.git-cliff
          pkgs.tea
        ];

        mkApp = drv: {
          type = "app";
          program = pkgs.lib.getExe drv;
        };

        mkReleasePlzOnCodeberg =
          subcommand:
          pkgs.writeShellApplication {
            name = subcommand;
            runtimeInputs = [
              ciToolchain
              pkgs.release-plz
              pkgs.gitMinimal
            ];
            text = ''
              release-plz ${subcommand} --forge gitea --git-token "$CODEBERG_TOKEN"
            '';
          };

        exportBuiltPathsHook = pkgs.writeShellApplication {
          name = "export-built-paths-hook";
          text = ''
            exported_paths_dir="$CI_WORKSPACE/.nix-built-paths"
            mkdir -p "$exported_paths_dir"
            read -ra built_paths <<< "$OUT_PATHS"
            nix copy --to "file://$exported_paths_dir?compression=zstd" "''${built_paths[@]}"
            printf '%s\n' "''${built_paths[@]}" >> "$exported_paths_dir/paths.txt"
          '';
        };

        buildAndExportPathsScript = pkgs.writeShellApplication {
          name = "build-and-export-paths";
          text = ''
            export NIX_CONFIG="''${NIX_CONFIG:-}
            post-build-hook = ${pkgs.lib.getExe exportBuiltPathsHook}"
            nix build --print-build-logs "$@"
          '';
        };

        uploadExportedPathsScript = pkgs.writeShellApplication {
          name = "upload-exported-paths";
          runtimeInputs = [ pkgs.cachix ];
          text = ''
            exported_paths_dir="$CI_WORKSPACE/.nix-built-paths"
            if [[ ! -s "$exported_paths_dir/paths.txt" ]]; then
              echo "No newly built paths to upload"
              exit 0
            fi
            mapfile -t built_paths < "$exported_paths_dir/paths.txt"
            nix copy --no-check-sigs --from "file://$exported_paths_dir" "''${built_paths[@]}"
            cachix push wvhulle "''${built_paths[@]}"
          '';
        };

        uploadReleaseAssetsScript = pkgs.writeShellApplication {
          name = "upload-release-assets";
          runtimeInputs = [
            pkgs.tea
            pkgs.coreutils
          ];
          text = ''
            cd "$1"
            sha256sum -- *.tar.gz > sha256sum.txt
            tea login add --name codeberg --url "$CI_FORGE_URL" --token "$CODEBERG_TOKEN"
            tea releases assets create --login codeberg --repo "$CI_REPO" "$CI_COMMIT_TAG" ./*
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
          recordings = recordings.all;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux (
          pkgs.lib.mapAttrs' (
            scene_name: scene_recording: pkgs.lib.nameValuePair "recording-${scene_name}" scene_recording
          ) recordings.scenes
        );

        checks = {
          inherit tests;
          fmt = rustfmtCrane.cargoFmt { inherit src; };
          ast-grep = astGrepCheck;
        };

        apps = {
          default = mkApp self.packages.${system}.default;
          cranelift = mkApp self.packages.${system}.cranelift;
          release = mkApp (mkReleasePlzOnCodeberg "release");
          release-pr = mkApp (mkReleasePlzOnCodeberg "release-pr");
          check-commits = mkApp checkCommitsScript;
          build-and-export-paths = mkApp buildAndExportPathsScript;
          upload-exported-paths = mkApp uploadExportedPathsScript;
          upload-release-assets = mkApp uploadReleaseAssetsScript;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          publish-recordings = mkApp recordings.publish-recordings;
        };

        devShells.default = nightlyCrane.devShell {
          packages = devTools;
          NU_SCRIPTS_DIR = "${nu-scripts}";
        };
      }
    );
}
