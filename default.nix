{ lib
, rustPlatform
, yarn
, prefetch-yarn-deps
, fetchYarnDeps
}:
let
  inherit (lib) fileset;
  lockFile = ./Cargo.lock;
  cargoFile = ./Cargo.toml;
  cargoToml = builtins.fromTOML (builtins.readFile cargoFile);

  yarnLock = ./tests/fixtures/schema_generation/yarn.lock;
  checkYarnDeps = fetchYarnDeps {
    name = "qlc-test-schema-generation-yarn-deps";
    inherit yarnLock;
    hash = "sha256-1KrnYGzkAE1f21q93DpknkHgq8l0V41MvoDeQFrFavM=";
  };
in
rustPlatform.buildRustPackage {
  pname = "qlc";
  inherit (cargoToml.package) version;

  src = fileset.toSource {
    root = ./.;
    fileset = fileset.intersection
      (fileset.gitTracked ./.)
      (fileset.unions [ lockFile cargoFile ./src ./tests ]);
  };
  cargoLock = { inherit lockFile; };

  preCheck = ''
    export HOME="$(mktemp -d)"
    pushd tests/fixtures/schema_generation

    yarn config set yarn-offline-mirror "${checkYarnDeps}"
    fixup-yarn-lock yarn.lock
    yarn install --offline --frozen-lockfile --no-progress --non-interactive

    yarn run build

    popd
  '';
  nativeCheckInputs = [ prefetch-yarn-deps yarn checkYarnDeps ];

  meta = {
    description = "A fun codegenerator for GraphQL clients.";
    homepage = "https://github.com/notarize/qlc";
    license = lib.licenses.mit;
    mainProgram = "qlc";
  };
}
