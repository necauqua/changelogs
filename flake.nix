{
  description = "Only a dev shell for now";
  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs = {
      nixpkgs.follows = "nixpkgs";
      flake-utils.follows = "flake-utils";
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.stable.latest.minimal.override {
              extensions = [
                "rust-src" # for rust-analyzer
                "clippy"
              ];
            })

            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.rustfmt))
            rust-analyzer
          ];
        };
      }
    );
}
