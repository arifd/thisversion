{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      fenix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-zm3dyIY2T414ZRR3EhLOvptzG6gta4WZUcawzMUWtqI=";
        };
        cargo-workspace-lints = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-workspace-lints";
          version = "0.1.4";
          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-ks3+S0LP9lEgjhvrq3s20gJnABPk7/sZdaPm/bDIMHo=";
          };
          cargoHash = "sha256-9SYhPY5LP7Lwytz+zr5RkQSdfBwQv/ELmloAky3bqZs=";
          doCheck = false;
        };
        cargo-semver-checks = pkgs.cargo-semver-checks.overrideAttrs {
          doCheck = false;
        };
      in
      {
        formatter = pkgs.nixfmt;
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            cargo-workspace-lints
            pkgs.bash
            pkgs.cargo-hack
            cargo-semver-checks
            pkgs.shellcheck
            pkgs.taplo
            pkgs.mdbook
            pkgs.ripgrep
          ];
        };
      }
    );
}
