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
          filter =
            path: _:
            !builtins.elem (baseNameOf path) [
              "demos"
              "xtask"
            ];
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

        xtaskArgs = {
          src = ciCrane.cleanCargoSource ./xtask;
          pname = "xtask";
          version = "0.0.0";
          strictDeps = true;
          cargoExtraArgs = "--locked";
        };
        xtaskDeps = ciCrane.buildDepsOnly xtaskArgs;
        xtaskPackage = ciCrane.buildPackage (
          xtaskArgs
          // {
            cargoArtifacts = xtaskDeps;
            doCheck = false;
            meta.mainProgram = "xtask";
          }
        );
        xtaskTests = ciCrane.cargoTest (xtaskArgs // { cargoArtifacts = xtaskDeps; });
        xtaskTool = import ./nix/xtask-tool.nix {
          inherit pkgs;
          xtask = xtaskPackage;
        };

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

        buildMacosPackage =
          macosArch:
          let
            macosTarget = "${macosArch}-apple-darwin";
            macosToolchain = pkgs.rust-bin.stable.latest.minimal.override { targets = [ macosTarget ]; };
            macosCrane = (crane.mkLib pkgs).overrideToolchain (_toolchainPkgs: macosToolchain);
            macosSdkRoot = pkgs.pkgsCross.aarch64-darwin.apple-sdk.src;

            macosArgs = {
              inherit src;
              strictDeps = true;
              pname = "nu-lint-macos";
              meta.mainProgram = "nu-lint";
              doCheck = false;
              dontStrip = true;
              cargoVendorDir = macosCrane.vendorCargoDeps {
                inherit src;
                overrideVendorCargoPackage =
                  lockedCrate: vendoredCrate:
                  if lockedCrate.name == "libproc" then
                    vendoredCrate.overrideAttrs {
                      postPatch = ''
                        substituteInPlace src/lib.rs \
                          --replace-fail '#[cfg(not(docsrs))]' '#[cfg(any())]' \
                          --replace-fail '#[cfg(docsrs)]' '#[cfg(all())]'
                      '';
                    }
                  else
                    vendoredCrate;
              };
              nativeBuildInputs = [
                pkgs.cargo-zigbuild
                pkgs.zig
              ];
              CARGO_BUILD_TARGET = macosTarget;
              MACOSX_DEPLOYMENT_TARGET = "11.0";
              SDKROOT = macosSdkRoot;
              "CARGO_TARGET_${pkgs.lib.toUpper (builtins.replaceStrings [ "-" ] [ "_" ] macosTarget)}_RUSTFLAGS" =
                "-C link-arg=-F${macosSdkRoot}/System/Library/Frameworks";
              cargoCheckCommand = "true";
              cargoBuildCommand = "cargoWithProfile zigbuild";
              preBuild = ''
                export HOME="$TMPDIR"
                export ZIG_GLOBAL_CACHE_DIR="$TMPDIR/zig-global-cache"
                export ZIG_LOCAL_CACHE_DIR="$TMPDIR/zig-local-cache"
              '';
            };
          in
          macosCrane.buildPackage (
            macosArgs
            // {
              cargoArtifacts = macosCrane.buildDepsOnly macosArgs;
              nativeBuildInputs = macosArgs.nativeBuildInputs ++ [ pkgs.rcodesign ];
              postFixup = ''
                rcodesign sign --binary-identifier nu-lint "$out/bin/nu-lint"
              '';
            }
          );

        macosPackages = {
          aarch64 = buildMacosPackage "aarch64";
          x86_64 = buildMacosPackage "x86_64";
        };

        macosReleaseArchives = pkgs.runCommand "nu-lint-dist-macos" { } (
          ''
            mkdir -p $out
          ''
          + pkgs.lib.concatMapAttrsStringSep "\n" (macosArch: macosPackage: ''
            tar -czf $out/nu-lint-${macosArch}-apple-darwin.tar.gz -C ${macosPackage}/bin nu-lint
          '') macosPackages
        );

        mkApp = drv: {
          type = "app";
          program = pkgs.lib.getExe drv;
        };

        mkXtaskApp = toolArgs: mkApp (xtaskTool toolArgs);

        exportBuiltPathsHook = xtaskTool {
          name = "export-built-paths-hook";
          runtimeInputs = [ ];
        };

        mkExportingBuildApp =
          name:
          mkXtaskApp {
            inherit name;
            runtimeInputs = [ ];
            wrapperArgs = [
              "--set"
              "EXPORT_HOOK_PROGRAM"
              (pkgs.lib.getExe exportBuiltPathsHook)
            ];
          };

        mkReleasePlzApp =
          name:
          mkXtaskApp {
            inherit name;
            runtimeInputs = [
              ciToolchain
              pkgs.release-plz
              pkgs.gitMinimal
            ];
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
          test-dependencies = devProfileDeps;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          x86_64-linux-musl = muslPackages.x86_64;
          aarch64-linux-musl = muslPackages.aarch64;
          dist = releaseArchives;
          aarch64-apple-darwin = macosPackages.aarch64;
          x86_64-apple-darwin = macosPackages.x86_64;
          dist-macos = macosReleaseArchives;
          recordings = recordings.all;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux (
          pkgs.lib.mapAttrs' (
            scene_name: scene_recording: pkgs.lib.nameValuePair "recording-${scene_name}" scene_recording
          ) recordings.scenes
        );

        checks = {
          inherit tests;
          xtask-tests = xtaskTests;
          fmt = rustfmtCrane.cargoFmt { inherit src; };
          ast-grep = astGrepCheck;
        };

        apps = {
          default = mkApp self.packages.${system}.default;
          cranelift = mkApp self.packages.${system}.cranelift;
          release = mkReleasePlzApp "release";
          release-pr = mkReleasePlzApp "release-pr";
          check-commits = mkXtaskApp {
            name = "check-commits";
            runtimeInputs = [
              pkgs.convco
              pkgs.gitMinimal
            ];
          };
          build-and-export-paths = mkExportingBuildApp "build-and-export-paths";
          build-unless-cached = mkExportingBuildApp "build-unless-cached";
          import-exported-paths = mkXtaskApp {
            name = "import-exported-paths";
            runtimeInputs = [ ];
          };
          upload-exported-paths = mkXtaskApp {
            name = "upload-exported-paths";
            runtimeInputs = [ pkgs.cachix ];
          };
          collect-release-archives = mkXtaskApp {
            name = "collect-release-archives";
            runtimeInputs = [ ];
          };
          upload-release-assets = mkXtaskApp {
            name = "upload-release-assets";
            runtimeInputs = [
              pkgs.tea
              pkgs.coreutils
            ];
          };
          update-dependencies = mkXtaskApp {
            name = "update-dependencies";
            runtimeInputs = [
              ciToolchain
              pkgs.cargo-edit
              pkgs.gitMinimal
              pkgs.tea
            ];
          };
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
