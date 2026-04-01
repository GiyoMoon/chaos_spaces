{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        src = pkgs.lib.cleanSourceWith {
          src = ./apps/server;
          filter = path: type: (pkgs.lib.hasSuffix ".sql" path) || (pkgs.lib.hasInfix "/.sqlx/" path) || (craneLib.filterCargoSources path type);
        };

        buildArgs = {
          inherit src;
          strictDeps = true;
        };
      in
      {
        packages.server = craneLib.buildPackage buildArgs;
      }
    );
}
