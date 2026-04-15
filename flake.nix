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
            laceSrc = pkgs.fetchFromGitHub {
              owner = "trolando";
              repo = "lace";
              rev = "v1.3.1";
              hash = "sha256-zd09+URUh2rxNeNcBid/KCTQ0wgRWWGq7qQ3m1AhcCo=";
            };
            sylvanSrc = pkgs.fetchFromGitHub {
              owner = "trolando";
              repo = "sylvan";
              rev = "v1.7.1";
              hash = "sha256-mca0j++dt/ehLRwikmz5EUaZ6XMA8F3k4RkknR+naM8=";
            };
            sylvanFetchContentToolchain = pkgs.writeText "sylvan-fetchcontent-toolchain.cmake" ''
              set(FETCHCONTENT_SOURCE_DIR_LACE "${laceSrc}" CACHE PATH "")
              set(FETCHCONTENT_SOURCE_DIR_SYLVAN "${sylvanSrc}" CACHE PATH "")
            '';

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
              nativeBuildInputs = [
                pkgs.cmake
                pkgs.git
                pkgs.pkg-config
              ];
              buildInputs = [ pkgs.gmp ];
              CMAKE_TOOLCHAIN_FILE = sylvanFetchContentToolchain;
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
        prismulti-tests = args.craneLib.cargoTest (
          args.commonArgs
          // {
            inherit (args) cargoArtifacts;
            RUST_TEST_THREADS = "1";
          }
        );
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
            args.pkgs.git
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
