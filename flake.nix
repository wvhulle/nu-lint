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

        toolchain = fenix.packages.${system}.default.toolchain;

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
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
          packages = [ toolchain ];
        };
      }
    );
}
