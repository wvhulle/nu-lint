{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      naersk,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Native build using nixpkgs stable Rust
        naersk' = pkgs.callPackage naersk { };

        # Fenix toolchain for dev shell only
        devToolchain = fenix.packages.${system}.latest.toolchain;

        # Native package for current system
        nativePackage = naersk'.buildPackage {
          src = ./.;
          meta.mainProgram = "nu-lint";
        };

        # Cross-compilation helper using fenix + naersk
        # Based on: https://github.com/nix-community/naersk/blob/master/examples/multi-target/flake.nix
        mkCrossPackage =
          target:
          let
            # Fenix toolchain with target's rust-std
            toolchain =
              with fenix.packages.${system};
              combine [
                minimal.cargo
                minimal.rustc
                targets.${target}.latest.rust-std
              ];

            naerskCross = naersk.lib.${system}.override {
              cargo = toolchain;
              rustc = toolchain;
            };

            # Cross-compilation toolchain from nixpkgs
            crossPkgs =
              pkgs.pkgsCross.${
                if target == "aarch64-unknown-linux-gnu" then
                  "aarch64-multiplatform"
                else
                  throw "Unsupported cross target: ${target}"
              };

            cc = crossPkgs.stdenv.cc;
            targetUpper = builtins.replaceStrings [ "-" ] [ "_" ] (pkgs.lib.toUpper target);
          in
          naerskCross.buildPackage {
            src = ./.;
            strictDeps = true;
            CARGO_BUILD_TARGET = target;
            depsBuildBuild = [ cc ];
            "CARGO_TARGET_${targetUpper}_LINKER" = "${cc}/bin/${cc.targetPrefix}cc";
            # Required by ring crate for cross-compiling assembly
            TARGET_CC = "${cc}/bin/${cc.targetPrefix}cc";
            meta.mainProgram = "nu-lint";
          };

      in
      {
        # Available packages (from x86_64-linux):
        #   nix build .#default              - Native x86_64-linux binary
        #   nix build .#x86_64-linux         - Same as default
        #   nix build .#aarch64-linux        - Cross-compiled ARM64 binary
        #
        # For cargo-binstall compatibility, binaries are named:
        #   nu-lint-x86_64-unknown-linux-gnu.tar.gz
        #   nu-lint-aarch64-unknown-linux-gnu.tar.gz
        # TODO: simplify based on https://github.com/cargo-bins/cargo-binstall/blob/main/SUPPORT.md
        packages = {
          # Default: native build for current system
          default = nativePackage;

          # Explicit architecture aliases
          x86_64-linux = nativePackage;
          aarch64-linux = mkCrossPackage "aarch64-unknown-linux-gnu";
        };

        apps.default = {
          type = "app";
          program = pkgs.lib.getExe self.packages.${system}.default;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl ];
          packages = [ devToolchain ];
        };
      }
    );
}
