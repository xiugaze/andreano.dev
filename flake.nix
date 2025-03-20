{
  description = "static site generator dev environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system: 
      let 
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # rustPlatform = pkgs.makeRustPlatform {
        #   cargo = pkgs.rust-bin.stable.latest.cargo;
        #   rustc = pkgs.rust-bin.stable.latest.rustc;
        # };


        andreano-dev-package = pkgs.rustPlatform.buildRustPackage {
          pname = "andreano-dev";
          version = "1.0";
          src = ./site/.;
          cargoLock = {
            lockFile = ./site/Cargo.lock;
          };
        };
    in
    {
      packages.default = andreano-dev-package;

      nixosModules.default = { config, lib, pkgs, ...}: 
        with lib;
        let 
          cfg = config.services.andreano-dev; 
        in {
          options.services.hello = {
            enable = mkEnableOption "hello service";
          };
      };
      devShells.default = with pkgs; mkShell.override { stdenv = clangStdenv; } {
          buildInputs = with pkgs; [
            llvmPackages.libclang
            llvmPackages.clang
          ];

          LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${llvmPackages.libclang.lib}/lib/clang/${lib.getVersion clang}/include";
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld"; 

          packages =
            with pkgs;
            let
              rust = pkgs.rust-bin.stable.latest.default.override {
                targets = [ "wasm32-unknown-unknown" ];
              };
            in
            [
              # rust
              rust
              rust-analyzer
              wasm-pack
              cmake
              python3
              sqlite
              lld

              cargo-flamegraph
              linuxKernel.packages.linux_latest_libre.perf

              # js
              bun

              # tools
              libwebp
            ];
        };
    });
}
