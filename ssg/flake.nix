{
  description = "static site generator dev environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      overlays = [ (import rust-overlay) ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs { inherit system overlays; };
          }
        );
    in
    {
      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default =
            with pkgs;
            mkShell.override { stdenv = clangStdenv; } {
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

                  # js
                  bun

                  # tools
                  libwebp
                ];
            };
        }
      );
    };
}
