{ pkgs, xtask }:
{
  name,
  taskArgs ? [ name ],
  runtimeInputs,
  wrapperArgs ? [ ],
}:
let
  inherit (pkgs) lib;
in
pkgs.runCommand name
  {
    nativeBuildInputs = [ pkgs.makeBinaryWrapper ];
    meta.mainProgram = name;
  }
  ''
    makeWrapper ${lib.getExe xtask} $out/bin/${name} \
      ${lib.concatMapStringsSep " " (taskArg: "--add-flag ${lib.escapeShellArg taskArg}") taskArgs} \
      ${lib.optionalString (runtimeInputs != [ ]) "--prefix PATH : ${lib.makeBinPath runtimeInputs}"} \
      ${lib.escapeShellArgs wrapperArgs}
  ''
