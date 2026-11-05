{
  pkgs,
  mkApp,
  cargoToolchain ? null,
  preCommitInputs ? [ ],
  extraReleaseInputs ? [ ],
  releaseWrapperArgs ? [ ],
}:
let
  inherit (pkgs) lib;

  cargoInputs = lib.optional (cargoToolchain != null) cargoToolchain;

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

  releasePlzApp =
    name:
    mkApp (nuScriptTool {
      inherit name;
      script = ../ci/release-tasks.nu;
      runtimeInputs =
        cargoInputs
        ++ [
          pkgs.release-plz
          pkgs.gitMinimal
        ]
        ++ extraReleaseInputs;
      wrapperArgs = releaseWrapperArgs;
    });

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
          --prefix PATH : ${lib.makeBinPath ([ pkgs.gitMinimal ] ++ preCommitInputs)}
      '';
in
releaseTaskApps
// {
  publish-unreleased-crate-version = releasePlzApp "publish-unreleased-crate-version";
  open-or-update-release-pr = releasePlzApp "open-or-update-release-pr";
  open-dependency-update-pr = mkApp (nuScriptTool {
    name = "open-dependency-update-pr";
    script = ../ci/dependency-updates.nu;
    runtimeInputs = cargoInputs ++ [
      pkgs.gitMinimal
      pkgs.tea
    ];
  });
  run-pre-commit-hooks = mkApp preCommitHooksTool;
  build-and-export-paths = exportingBuildApp "build-and-export-paths";
  build-unless-cached = exportingBuildApp "build-unless-cached";
  import-exported-paths = mkApp (nixBuiltPathsTool {
    name = "import-exported-paths";
  });
  copy-build-output = mkApp (nixBuiltPathsTool {
    name = "copy-build-output";
  });
  upload-exported-paths = mkApp (nixBuiltPathsTool {
    name = "upload-exported-paths";
    runtimeInputs = [ pkgs.cachix ];
  });
}
