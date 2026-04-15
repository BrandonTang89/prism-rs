{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      crane,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };
            rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
            patchedCudd = pkgs.cudd.overrideAttrs (old: {
              patches = (old.patches or [ ]) ++ [ ./vendor/cudd-sys/patches/cudd-add-max-min.patch ];
            });

            src =
              let
                nonStandardFilter =
                  path: type:
                  let
                    baseName = baseNameOf path;
                  in
                  (pkgs.lib.hasInfix "/docs/" path)
                  || (pkgs.lib.hasInfix "/tests/dtmc/" path)
                  || (pkgs.lib.hasSuffix ".md" baseName)
                  || (pkgs.lib.hasSuffix ".patch" baseName)
                  || (pkgs.lib.hasSuffix ".prism" baseName)
                  || (pkgs.lib.hasSuffix ".prop" baseName)
                  || (pkgs.lib.hasSuffix ".lalrpop" baseName);
              in
              pkgs.lib.cleanSourceWith {
                src = ./.;
                filter = path: type: (nonStandardFilter path type) || (craneLib.filterCargoSources path type);
              };

            commonArgs = {
              inherit src;
              strictDeps = true;
              cargoExtraArgs = "--no-default-features";
              nativeBuildInputs = [ pkgs.pkg-config ];
              buildInputs = [ patchedCudd ];
              CARGO_BUILD_RUSTFLAGS = [
                "-L"
                "${patchedCudd}/lib"
                "-l"
                "static=cudd"
              ];
            };

            cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          in
          f {
            inherit
              pkgs
              rustToolchain
              craneLib
              commonArgs
              cargoArtifacts
              src
              ;
          }
        );
    in
    {
      packages = forAllSystems (args: {
        default = args.craneLib.buildPackage (args.commonArgs // { inherit (args) cargoArtifacts; });
      });

      checks = forAllSystems (args: {
        prismulti-tests = args.craneLib.cargoTest (args.commonArgs // { inherit (args) cargoArtifacts; });
        prismulti-fmt = args.craneLib.cargoFmt { inherit (args) src; };
        prismulti-clippy = args.craneLib.cargoClippy (
          args.commonArgs
          // {
            inherit (args) cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          }
        );
      });

      devShells = forAllSystems (args: {
        default = args.pkgs.mkShell {
          packages = [
            args.rustToolchain
            args.pkgs.python3
            args.pkgs.python3Packages.mypy
            args.pkgs.uv
            args.pkgs.graphviz
            args.pkgs.pkg-config
            args.pkgs.gnuplot
            args.pkgs.cmake
            args.pkgs.gcc15
            args.pkgs.gmp
            args.pkgs.hyperfine
          ]
          ++ args.pkgs.lib.optionals args.pkgs.stdenv.isLinux [
            args.pkgs.perf
          ];
        };
      });
    };
}
