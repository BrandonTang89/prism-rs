{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rust = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = [
              rust
              pkgs.rust-analyzer
              pkgs.python3
              pkgs.python3Packages.mypy
              pkgs.uv
              pkgs.graphviz
            ];
          };
        });
    };
}
