{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    fenix.url = "github:nix-community/fenix";
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

        # Use nixpkgs stable Rust for building (no fenix dependency in output)
        naersk' = pkgs.callPackage naersk { };

        # Fenix toolchain only for dev shell
        devToolchain = fenix.packages.${system}.latest.toolchain;
      in
      {
        packages.default = naersk'.buildPackage {
          src = ./.;
          meta.mainProgram = "nu-lint";
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
