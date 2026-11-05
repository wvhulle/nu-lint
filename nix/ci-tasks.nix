{ pkgs, mkApp }:
let
  inherit (pkgs) lib;

  ciToolchain = pkgs.rust-bin.stable.latest.minimal;
  rustfmtToolchain = pkgs.rust-bin.selectLatestNightlyWith (
    toolchain: toolchain.minimal.override { extensions = [ "rustfmt" ]; }
  );

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
        makeWrapper ${lib.getExe pkgs.nushell} $out/bin/${name} \
          --add-flags ${script} \
          --add-flags ${name} \
          ${lib.optionalString (runtimeInputs != [ ]) "--prefix PATH : ${lib.makeBinPath runtimeInputs}"} \
          ${lib.escapeShellArgs wrapperArgs}
      '';

  nixBuiltPathsTool = toolArgs: nuScriptTool (toolArgs // { script = ../ci/nix-built-paths.nu; });

  exportBuiltPathsHook = nixBuiltPathsTool { name = "export-built-paths-hook"; };

  exportingBuildApp =
    name:
    mkApp (nixBuiltPathsTool {
      inherit name;
      wrapperArgs = [
        "--set"
        "EXPORT_HOOK_PROGRAM"
        (lib.getExe exportBuiltPathsHook)
      ];
    });

  releasePlzInputs = [
    ciToolchain
    pkgs.release-plz
    pkgs.gitMinimal
  ];

  releaseTaskApps =
    lib.mapAttrs
      (
        name: runtimeInputs:
        mkApp (nuScriptTool {
          inherit name runtimeInputs;
          script = ../ci/release-tasks.nu;
        })
      )
      {
        publish-unreleased-crate-version = releasePlzInputs;
        open-or-update-release-pr = releasePlzInputs;
        authenticate-git-origin = [ pkgs.gitMinimal ];
        collect-release-archives = [ ];
        upload-release-assets = [ pkgs.tea ];
        check-commit-messages = [
          pkgs.convco
          pkgs.gitMinimal
        ];
      };

  preCommitHooksTool =
    pkgs.runCommand "run-pre-commit-hooks"
      {
        nativeBuildInputs = [ pkgs.makeBinaryWrapper ];
        meta.mainProgram = "run-pre-commit-hooks";
      }
      ''
        makeWrapper ${lib.getExe pkgs.prek} $out/bin/run-pre-commit-hooks \
          --add-flags "run --all-files --show-diff-on-failure" \
          --prefix PATH : ${
            lib.makeBinPath [
              pkgs.ast-grep
              pkgs.gitMinimal
              rustfmtToolchain
            ]
          }
      '';
in
releaseTaskApps
// {
  run-pre-commit-hooks = mkApp preCommitHooksTool;
  build-and-export-paths = exportingBuildApp "build-and-export-paths";
  build-unless-cached = exportingBuildApp "build-unless-cached";
  import-exported-paths = mkApp (nixBuiltPathsTool {
    name = "import-exported-paths";
  });
  upload-exported-paths = mkApp (nixBuiltPathsTool {
    name = "upload-exported-paths";
    runtimeInputs = [ pkgs.cachix ];
  });
}
