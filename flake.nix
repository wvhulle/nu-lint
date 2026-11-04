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

        nuScriptTool =
          {
            script,
            name,
            runtimeInputs ? [ ],
            wrapperArgs ? [ ],
          }:
          pkgs.runCommand name
            {
              nativeBuildInputs = [ pkgs.makeBinaryWrapper ];
              meta.mainProgram = name;
            }
            ''
              makeWrapper ${pkgs.lib.getExe pkgs.nushell} $out/bin/${name} \
                --add-flags ${script} \
                --add-flags ${name} \
                ${
                  pkgs.lib.optionalString (
                    runtimeInputs != [ ]
                  ) "--prefix PATH : ${pkgs.lib.makeBinPath runtimeInputs}"
                } \
                ${pkgs.lib.escapeShellArgs wrapperArgs}
            '';

        nixBuiltPathsTool = toolArgs: nuScriptTool (toolArgs // { script = ./ci/nix-built-paths.nu; });
        releaseTaskApp = toolArgs: mkApp (nuScriptTool (toolArgs // { script = ./ci/release-tasks.nu; }));

        ciTaskTests = pkgs.runCommand "nu-lint-ci-task-tests" { nativeBuildInputs = [ pkgs.nushell ]; } ''
          nu ${./ci}/test-ci-tasks.nu
          touch $out
        '';

        releasePlzInputs = [
          ciToolchain
          pkgs.release-plz
          pkgs.gitMinimal
        ];

        exportBuiltPathsHook = nixBuiltPathsTool { name = "export-built-paths-hook"; };

        exportingBuildTool =
          toolArgs:
          nixBuiltPathsTool (
            toolArgs
            // {
              wrapperArgs = [
                "--set"
                "EXPORT_HOOK_PROGRAM"
                (pkgs.lib.getExe exportBuiltPathsHook)
              ];
            }
          );

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
          ci-task-tests = ciTaskTests;
          fmt = rustfmtCrane.cargoFmt { inherit src; };
          ast-grep = astGrepCheck;
        };

        apps = {
          default = mkApp self.packages.${system}.default;
          cranelift = mkApp self.packages.${system}.cranelift;
          release = releaseTaskApp {
            name = "release";
            runtimeInputs = releasePlzInputs;
          };
          release-pr = releaseTaskApp {
            name = "release-pr";
            runtimeInputs = releasePlzInputs;
          };
          authenticate-git-origin = releaseTaskApp {
            name = "authenticate-git-origin";
            runtimeInputs = [ pkgs.gitMinimal ];
          };
          collect-release-archives = releaseTaskApp { name = "collect-release-archives"; };
          upload-release-assets = releaseTaskApp {
            name = "upload-release-assets";
            runtimeInputs = [ pkgs.tea ];
          };
          check-commits = releaseTaskApp {
            name = "check-commits";
            runtimeInputs = [
              pkgs.convco
              pkgs.gitMinimal
            ];
          };
          build-and-export-paths = mkApp (exportingBuildTool {
            name = "build-and-export-paths";
          });
          build-unless-cached = mkApp (exportingBuildTool {
            name = "build-unless-cached";
          });
          import-exported-paths = mkApp (nixBuiltPathsTool {
            name = "import-exported-paths";
          });
          upload-exported-paths = mkApp (nixBuiltPathsTool {
            name = "upload-exported-paths";
            runtimeInputs = [ pkgs.cachix ];
          });
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
