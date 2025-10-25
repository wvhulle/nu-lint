{
  lib,
  rustPlatform,
  fetchFromGitHub,
}:

rustPlatform.buildRustPackage {
  pname = "nu-lint";
  version = "0.0.7";

  src = lib.cleanSource ./.;

  cargoHash = "sha256-DiExIO0NG2GqvvXGpHsoNGBt8BTBn533SKC+mkXo2F4=";

  nativeBuildInputs = [ ];

  buildInputs = [ ];

  # Build configuration
  doCheck = false; # Skip tests during build

  # Enable unstable features for nu-glob compatibility
  RUSTC_BOOTSTRAP = "1";

  meta = with lib; {
    description = "A linter for Nushell scripts";
    homepage = "https://github.com/wvhulle/nu-lint";
    license = licenses.mit;
    maintainers = [ ];
    mainProgram = "nu-lint";
    platforms = platforms.all;
  };
}
