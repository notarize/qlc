{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      systems = {
        aarch64-darwin = "aarch64-apple-darwin";
        aarch64-linux = "aarch64-unknown-linux-musl";
        x86_64-darwin = "x86_64-apple-darwin";
        x86_64-linux = "x86_64-unknown-linux-musl";
      };

      eachSystem = flakeFunc: lib.foldlAttrs
        (accum: system: rustTargetTriple: lib.recursiveUpdate accum (flakeFunc system))
        { }
        systems;
    in
    eachSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        defaultQlcPackage = self.packages.${system}.${"qlc-${system}"};

        mixinQlcPackage = accum: targetSystem: rustTargetTriple:
          let
            mkQlcPackage = p: { "qlc-${targetSystem}" = p.callPackage (import ./default.nix) { }; };
            isTargetLinux = lib.hasSuffix "-linux" targetSystem;
            crossPkgs = import nixpkgs {
              inherit system;
              crossSystem = {
                config = rustTargetTriple;
                rustc.config = rustTargetTriple;
                isStatic = isTargetLinux;
              };
            };
          in
          # Not a cross compile, just use "native nixpkgs"
          if targetSystem == system then
            accum // (mkQlcPackage pkgs)
          # Some combinations of cross-compile are not supported
          else if isTargetLinux then
            accum // (mkQlcPackage crossPkgs)
          else accum;
      in
      {
        devShells.${system}.default = pkgs.callPackage (import ./shell.nix) { qlc = defaultQlcPackage; };

        formatter.${system} = pkgs.nixpkgs-fmt;

        packages.${system} = lib.foldlAttrs mixinQlcPackage { default = defaultQlcPackage; } systems;
      }
    );
}
