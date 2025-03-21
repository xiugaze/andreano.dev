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


        # define the server package
        andreano-dev-package = pkgs.rustPlatform.buildRustPackage {
          pname = "andreano-dev";
          version = "1.0";
          src = ./site/.;
          cargoLock = {
            lockFile = ./site/Cargo.lock;
          };
        };

    in {
      packages.default = andreano-dev-package;
      packages.andreano-dev-package = andreano-dev-package;
      packages.andreano-dev-site = pkgs.runCommand "andreano-dev-site" {
        buildInputs = [ self.packages.${system}.default ];
      } ''
          mkdir -p $out
          cp -r ${./site/website} ./website
          chmod -R u+w ./website
          cd ./website
          ${self.packages.${system}.default}/bin/site
          cp -r . $out/  # Copy the entire contents of ./website to $out
        '';


      nixosModules.default = { config, lib, pkgs, ...}:
          let 
            cfg = config.services.andreano-dev;
          in with lib; {
        options = {
          services.andreano-dev = {
            enable = mkOption {
              default = false;
              description = "enable the web server";
            } ;
            user = mkOption {
              default = "caleb";
              description = "user to run the service";
            };
          };
        };

        config = mkIf config.services.andreano-dev.enable {
          systemd.services.andreano-dev = let
            package-base = "${self.packages.${pkgs.system}.default}";
            serve-path = "${self.packages.${pkgs.system}.andreano-dev-site}/static";
          in {
            wantedBy = [ "default.target" ]; 
            after = [ "network.target" ];
            serviceConfig = {
              Type = "simple";
              User = "${cfg.user}";
              ExecStart = "${package-base}/bin/site serve ${serve-path}";
            };
          };
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
