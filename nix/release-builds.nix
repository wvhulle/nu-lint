{
  pkgs,
  crane,
  src,
}:
let
  inherit (pkgs) lib;

  sharedReleaseArgs = {
    inherit src;
    strictDeps = true;
    meta.mainProgram = "nu-lint";
    doCheck = false;
  };

  cargoTargetEnvName = cargoTarget: builtins.replaceStrings [ "-" ] [ "_" ] cargoTarget;

  buildMuslPackage =
    muslArch:
    let
      muslTarget = "${muslArch}-unknown-linux-musl";
      muslPkgs =
        if muslArch == "x86_64" then pkgs.pkgsCross.musl64 else pkgs.pkgsCross.aarch64-multiplatform-musl;
      muslToolchain = pkgs.rust-bin.stable.latest.default.override { targets = [ muslTarget ]; };
      muslCrane = (crane.mkLib pkgs).overrideToolchain muslToolchain;

      muslCompiler = "${muslPkgs.stdenv.cc}/bin/${muslPkgs.stdenv.cc.targetPrefix}cc";
      muslArchiver = "${muslPkgs.stdenv.cc.bintools}/bin/${muslPkgs.stdenv.cc.targetPrefix}ar";
      muslTargetEnv = cargoTargetEnvName muslTarget;

      muslArgs = sharedReleaseArgs // {
        pname = "nu-lint-static";
        CARGO_BUILD_TARGET = muslTarget;
        "CARGO_TARGET_${lib.toUpper muslTargetEnv}_LINKER" = muslCompiler;
        "CC_${muslTargetEnv}" = muslCompiler;
        "AR_${muslTargetEnv}" = muslArchiver;
      };
    in
    muslCrane.buildPackage (muslArgs // { cargoArtifacts = muslCrane.buildDepsOnly muslArgs; });

  buildMacosPackage =
    macosArch:
    let
      macosTarget = "${macosArch}-apple-darwin";
      macosToolchain = pkgs.rust-bin.stable.latest.minimal.override { targets = [ macosTarget ]; };
      macosCrane = (crane.mkLib pkgs).overrideToolchain macosToolchain;
      macosSdkRoot = pkgs.pkgsCross.aarch64-darwin.apple-sdk.src;

      macosArgs = sharedReleaseArgs // {
        pname = "nu-lint-macos";
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
        nativeBuildInputs = [ pkgs.cargo-zigbuild ];
        CARGO_ZIGBUILD_ZIG_COMMAND = lib.getExe pkgs.zig_0_15;
        CARGO_BUILD_TARGET = macosTarget;
        MACOSX_DEPLOYMENT_TARGET = "11.0";
        SDKROOT = macosSdkRoot;
        "CARGO_TARGET_${lib.toUpper (cargoTargetEnvName macosTarget)}_RUSTFLAGS" =
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

  releaseArchives =
    {
      name,
      targetSuffix,
      packagesByArch,
    }:
    pkgs.runCommand name { } (
      ''
        mkdir -p $out
      ''
      + lib.concatMapAttrsStringSep "\n" (releaseArch: releasePackage: ''
        tar -czf $out/nu-lint-${releaseArch}-${targetSuffix}.tar.gz -C ${releasePackage}/bin nu-lint
      '') packagesByArch
    );

  muslPackages = {
    x86_64 = buildMuslPackage "x86_64";
    aarch64 = buildMuslPackage "aarch64";
  };

  macosPackages = {
    aarch64 = buildMacosPackage "aarch64";
    x86_64 = buildMacosPackage "x86_64";
  };
in
{
  x86_64-linux-musl = muslPackages.x86_64;
  aarch64-linux-musl = muslPackages.aarch64;
  dist = releaseArchives {
    name = "nu-lint-dist";
    targetSuffix = "unknown-linux-musl";
    packagesByArch = muslPackages;
  };
  aarch64-apple-darwin = macosPackages.aarch64;
  x86_64-apple-darwin = macosPackages.x86_64;
  dist-macos = releaseArchives {
    name = "nu-lint-dist-macos";
    targetSuffix = "apple-darwin";
    packagesByArch = macosPackages;
  };
}
