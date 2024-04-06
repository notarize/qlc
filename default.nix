{ lib
, rustPlatform
}:
let
  inherit (lib) fileset;
  lockFile = ./Cargo.lock;
  cargoFile = ./Cargo.toml;
  cargoToml = builtins.fromTOML (builtins.readFile cargoFile);
in
rustPlatform.buildRustPackage {
  pname = "qlc";
  inherit (cargoToml.package) version;

  src = fileset.toSource {
    root = ./.;
    fileset = fileset.intersection
      (fileset.gitTracked ./.)
      (fileset.unions [ lockFile cargoFile ./src ]);
  };
  cargoLock = { inherit lockFile; };

  doCheck = false;

  meta = {
    description = "A fun codegenerator for GraphQL clients.";
    homepage = "https://github.com/notarize/qlc";
    license = lib.licenses.mit;
    mainProgram = "qlc";
  };
}
