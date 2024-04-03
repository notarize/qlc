{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      eachSystem = flakeFunc: builtins.foldl'
        (accum: system: lib.recursiveUpdate accum (flakeFunc system))
        { }
        [
          "aarch64-darwin"
          "aarch64-linux"
          "x86_64-darwin"
          "x86_64-linux"
        ];
    in
    eachSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        defaultQlcPackage = pkgs.callPackage (import ./default.nix) { };
      in
      {
        devShells.${system}.default = pkgs.callPackage (import ./shell.nix) { qlc = defaultQlcPackage; };

        formatter.${system} = pkgs.nixpkgs-fmt;

        packages.${system}.default = defaultQlcPackage;
      }
    );
}
