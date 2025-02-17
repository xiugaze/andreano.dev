{
  description = "static site generator dev environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      overlays = [ (import rust-overlay) ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs { inherit system overlays; };
      });
    in {
      devShells = forEachSupportedSystem ({ pkgs }: {
        default = with pkgs; mkShell.override { stdenv = clangStdenv; } {
          buildInputs = with pkgs; [
            llvmPackages.libclang
            llvmPackages.clang
          ];

          LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${llvmPackages.libclang.lib}/lib/clang/${lib.getVersion clang}/include";

          packages = with pkgs; [
            # rust 
            rust-bin.stable.latest.default
            rust-analyzer

            cmake
          ];
        };
      });
    };
}
