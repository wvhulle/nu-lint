{
  pkgs,
  crane,
  src,
}:
let
  stableToolchain = pkgs.rust-bin.stable.latest.default;
  ciToolchain = pkgs.rust-bin.stable.latest.minimal;
  nightlyToolchain = pkgs.rust-bin.nightly.latest.default.override {
    extensions = [ "rustc-codegen-cranelift-preview" ];
  };

  stableCrane = (crane.mkLib pkgs).overrideToolchain stableToolchain;
  ciCrane = (crane.mkLib pkgs).overrideToolchain ciToolchain;
  nightlyCrane = (crane.mkLib pkgs).overrideToolchain nightlyToolchain;

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
  testDependencies = ciCrane.buildDepsOnly devProfileArgs;
  testSuite = ciCrane.cargoTest (
    devProfileArgs
    // {
      cargoArtifacts = testDependencies;
      doInstallCargoArtifacts = false;
      cargoTestExtraArgs = "--workspace";
    }
  );

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
in
{
  inherit
    llvmDeps
    llvmPackage
    xtaskPackage
    testDependencies
    testSuite
    craneliftDeps
    craneliftPackage
    nightlyCrane
    ;
}
