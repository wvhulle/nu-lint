{
  pkgs,
  nu-lint,
  xtask,
}:
let
  inherit (pkgs) lib;

  xtaskTool = import ../nix/xtask-tool.nix { inherit pkgs xtask; };

  record-scene = xtaskTool {
    name = "record-scene";
    runtimeInputs = [
      nu-lint
      pkgs.bashInteractive
      pkgs.ffmpeg
      pkgs.helix
      pkgs.vhs
    ];
    wrapperArgs = [
      "--set"
      "FONTCONFIG_FILE"
      (pkgs.makeFontsConf { fontDirectories = [ pkgs.jetbrains-mono ]; })
      "--set"
      "TZDIR"
      "${pkgs.tzdata}/share/zoneinfo"
      "--set"
      "LC_ALL"
      "C.UTF-8"
      "--set"
      "SHELL"
      (lib.getExe pkgs.bashInteractive)
      "--set"
      "VHS_NO_SANDBOX"
      "true"
    ];
  };

  sceneNames = lib.pipe (builtins.readDir ./.) [
    (lib.filterAttrs (
      file: kind:
      kind == "regular" && lib.hasSuffix ".tape" file && file != "common.tape" && file != "open.tape"
    ))
    builtins.attrNames
    (map (lib.removeSuffix ".tape"))
  ];

  recordScene =
    scene:
    pkgs.stdenvNoCC.mkDerivation {
      name = "nu-lint-recording-${scene}";
      src = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          ./.helix
          ./common.tape
          ./open.tape
          ./sample_script.nu
          (./. + "/${scene}.tape")
        ];
      };
      nativeBuildInputs = [ record-scene ];
      buildPhase = "HOME=$TMPDIR record-scene ${scene}.tape --out $out";
      dontInstall = true;
    };

  scenes = lib.genAttrs sceneNames recordScene;
in
{
  inherit scenes;
  all = pkgs.symlinkJoin {
    name = "nu-lint-recordings";
    paths = builtins.attrValues scenes;
  };
  publish-recordings = xtaskTool {
    name = "publish-recordings";
    runtimeInputs = [
      pkgs.git
      pkgs.jujutsu
      pkgs.openssh
    ];
  };
}
