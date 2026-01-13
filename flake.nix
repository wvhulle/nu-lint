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

        # Fenix toolchain for dev shell
        devToolchain = fenix.packages.${system}.latest.toolchain;

        # Cross-compilation helper
        # Source: https://github.com/nix-community/fenix
        mkCrossPackage = target: let
          toolchain = with fenix.packages.${system};
            combine [
              minimal.cargo
              minimal.rustc
              targets.${target}.latest.rust-std
            ];
          naerskCross = naersk.lib.${system}.override {
            cargo = toolchain;
            rustc = toolchain;
          };
          # Get the cross-compilation toolchain for linking
          crossPkgs = pkgs.pkgsCross.${
            if target == "aarch64-unknown-linux-gnu" then "aarch64-multiplatform"
            else if target == "x86_64-unknown-linux-gnu" then "gnu64"
            else throw "Unsupported cross target: ${target}"
          };
          cc = crossPkgs.stdenv.cc;
          linkerEnvVar = "CARGO_TARGET_${builtins.replaceStrings ["-"] ["_"] (pkgs.lib.toUpper target)}_LINKER";
        in
          naerskCross.buildPackage {
            src = ./.;
            CARGO_BUILD_TARGET = target;
            "${linkerEnvVar}" = "${cc}/bin/${cc.targetPrefix}cc";
            meta.mainProgram = "nu-lint";
          };
      in
      {
        packages = {
          default = naersk'.buildPackage {
            src = ./.;
            meta.mainProgram = "nu-lint";
          };

          # Cross-compiled packages (from x86_64-linux)
          # Usage: nix build .#aarch64-linux
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
