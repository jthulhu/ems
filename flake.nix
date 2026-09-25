{
  description = "inv";

  inputs = {
    crane.url = "github:ipetkov/crane";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, utils, crane, rust-overlay, advisory-db }:
    utils.lib.eachDefaultSystem (system:
      let
        inherit (crane-lib) cleanCargoSource crateNameFromCargoToml;
        inherit (crane-lib.fileset) commonCargoSources;
        inherit (pkgs.lib) fileset makeLibraryPath;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        crane-lib = crane.mkLib pkgs;
        src = cleanCargoSource ./.;
        common-args = {
          inherit src;
          strictDeps = true;
        };
        cargo-artifacts = crane-lib.buildDepsOnly (common-args // { pname = "inv"; });
        files-of = crate: fileset.toSource {
          root = ./.;
          fileset = fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            (commonCargoSources crate)
          ];
        };
        per-crate-args = name: common-args // {
          inherit (crateNameFromCargoToml { src = files-of ./${name}; }) version pname;
          cargoArtifacts = cargo-artifacts;
          cargoExtraArgs = "-p ${name}";
          src = files-of ./${name};
        };

        # Packages
      in {
        packages = {
          inherit cargo-artifacts;
        };
        checks = {
          clippy = crane-lib.cargoClippy (common-args // {
            cargoArtifacts = cargo-artifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          doc = crane-lib.cargoDoc (common-args // {
            cargoArtifacts = cargo-artifacts;
            env.RUSTDOCFLAGS = "--deny warnings";
          });
          
          format = crane-lib.cargoFmt common-args;
        };
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            gpgme
            libsodium
            sqlcipher
          ];
          buildInputs = with pkgs; [
            gdb
            rust
            cargo
            cargo-edit
            cargo-audit
            cargo-deny
            cargo-machete
            rustfmt
            clippy
            rust-analyzer
            crates-lsp
          ];
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${makeLibraryPath (with pkgs; [ wayland libGL libxkbcommon ])}"
          '';
        };
      });
}
