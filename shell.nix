{ mkShell
, qlc
, clippy
, rustfmt
, just
, yarn
, writeText
, runCommand
, makeBinaryWrapper
}:
let
  justfile = writeText "qlc-justfile" ''
    [private]
    default:
      @just --list --list-heading $'QLC recipes\n'

    # Build the schema for cargo tests
    build-test-schema:
      @yarn --cwd tests/fixtures/schema_generation install --frozen-lockfile
      # Turning tests/fixtures/schema_generation/schema.graphl into a usable tests/fixtures/schema_generation/output/schema.json
      @yarn --cwd tests/fixtures/schema_generation run build
    
    # Run tests
    test *args:
      cargo test {{args}}

    # Format source files
    format:
      cargo fmt --all
      nix fmt

    # Lint Rust source files
    lint:
      cargo clippy --all-targets --all-features -- -D warnings
  '';
  justWithConfig = runCommand
    "qlc-wrapped-just"
    { nativeBuildInputs = [ makeBinaryWrapper ]; }
    ''
      makeBinaryWrapper ${just}/bin/just $out/bin/just \
        --add-flags '--justfile ${justfile}' \
        --add-flags '--working-directory .'
    '';
in
mkShell {
  name = "qlc-devshell";
  inputsFrom = [ qlc ];
  packages = [
    yarn
    clippy
    rustfmt
    justWithConfig
  ];
}
