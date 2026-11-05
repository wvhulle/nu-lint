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
        inherit (pkgs) lib;

        src = lib.cleanSourceWith {
          src = lib.cleanSource ./.;
          filter = path: _: baseNameOf path != "demos";
        };

        mkApp = drv: {
          type = "app";
          program = lib.getExe drv;
        };

        nativeBuilds = import ./nix/native-builds.nix { inherit pkgs crane src; };
        releaseBuilds = import ./nix/release-builds.nix { inherit pkgs crane src; };
        ciTaskApps = import ./nix/ci-tasks.nix {
          inherit pkgs mkApp;
          cargoToolchain = pkgs.rust-bin.stable.latest.minimal;
          preCommitInputs = [
            pkgs.ast-grep
            (pkgs.rust-bin.selectLatestNightlyWith (
              toolchain: toolchain.minimal.override { extensions = [ "rustfmt" ]; }
            ))
          ];
        };
        recordings = import ./demos {
          inherit pkgs;
          nu-lint = nativeBuilds.llvmPackage;
          xtask = nativeBuilds.xtaskPackage;
        };
      in
      {
        packages = {
          default = nativeBuilds.llvmPackage;
          llvm = nativeBuilds.llvmPackage;
          cranelift = nativeBuilds.craneliftPackage;
          deps = nativeBuilds.llvmDeps;
          deps-cranelift = nativeBuilds.craneliftDeps;
          test-dependencies = nativeBuilds.testDependencies;
        }
        // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux (
          releaseBuilds
          // {
            recordings = recordings.all;
          }
          // lib.mapAttrs' (
            scene_name: scene_recording: lib.nameValuePair "recording-${scene_name}" scene_recording
          ) recordings.scenes
        );

        checks.tests = nativeBuilds.testSuite;

        apps =
          ciTaskApps
          // {
            default = mkApp nativeBuilds.llvmPackage;
            cranelift = mkApp nativeBuilds.craneliftPackage;
          }
          // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            publish-recordings = mkApp recordings.publish-recordings;
            publish-recordings-to-codeberg = mkApp recordings.publish-recordings-to-codeberg;
          };

        devShells.default = nativeBuilds.nightlyCrane.devShell {
          packages = [
            pkgs.ast-grep
            pkgs.cargo-edit
            pkgs.prek
            pkgs.convco
            pkgs.release-plz
            pkgs.git-cliff
            pkgs.tea
          ];
          NU_SCRIPTS_DIR = "${nu-scripts}";
        };
      }
    );
}
