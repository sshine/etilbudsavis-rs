# The build definition, shared by this flake's packages and by the overlay.
#
# Taking `pkgs` as an argument (rather than closing over this flake's own) is what
# lets the overlay build against the consumer's nixpkgs, so downstream can override
# and cross-compile it.
#
# `crate` selects the workspace member. A library member installs no binary, so
# mainProgram is only set for the executable members.
{
  lib,
  rustPlatform,
  cacert,
  crate ? "etilbudsavis-cli",
  ...
}:
let
  workspace = lib.importTOML ../Cargo.toml;
  manifest = lib.importTOML (../crates + "/${crate}/Cargo.toml");
in
rustPlatform.buildRustPackage {
  pname = crate;
  version = workspace.workspace.package.version;

  # reqwest with rustls loads the system trust store when a client is built.
  # Mock tests only talk to loopback over plain HTTP; this gets past construction.
  SSL_CERT_FILE = "${cacert}/etc/ssl/certs/ca-bundle.crt";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../crates
      ../README.md
    ];
  };
  cargoLock.lockFile = ../Cargo.lock;

  cargoBuildFlags = [
    "--package"
    crate
  ];

  cargoTestFlags = [
    "--package"
    crate
  ];

  meta = {
    inherit (manifest.package) description;
    license = with lib.licenses; [
      mit
      asl20
    ];
  }
  // lib.optionalAttrs (crate != "etilbudsavis-api") {
    mainProgram = crate;
  };
}
