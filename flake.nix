{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    solc = {
      url = "github:hellwolf/solc.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crate2nix = {
      url = "github:nix-community/crate2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, solc, crate2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            solc.overlay
            (final: prev: {
              solc = (solc.mkDefault final final.solc_0_8_27);
            })
          ];
        };
        lib = pkgs.lib;
        toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rustfmt" "clippy" "rust-src" ];
        };
        crate2nix-tools = crate2nix.tools.${system};
        generated = crate2nix-tools.generatedCargoNix {
          src = ./.;
          name = "foundry";
        };
        buildRustCrateForPkgs = pkgs: with pkgs; buildRustCrate.override {
          defaultCrateOverrides = defaultCrateOverrides // {
            chisel = attrs: {
              # The subcrates don't get the full workspace source, so we need to
              # update this path.
              prePatch = (attrs.prePatch or "") + ''
                substituteInPlace \
                  src/session_source.rs \
                  --replace-fail \
                  '../../../testdata/cheats/Vm.sol' \
                  '${./testdata/cheats/Vm.sol}'
              '';
            };
            svm-rs-builds = attrs: {
              # Why is this even a thing? It seems silly to generate the list of
              # releases at build time.
              # Force svm-rs-builds to generate in offline mode and see how
              # much breaks.
              features = (attrs.features or [ ]) ++ [ "_offline" ];
            };
          };
          cargo = toolchain;
          rustc = toolchain;
        };
        cargoWorkspace = pkgs.callPackage generated {
          inherit buildRustCrateForPkgs;
        };
        crateBuilds = lib.mapAttrs (_: value: value.build) cargoWorkspace.workspaceMembers;
        # all-in-one derivation that also includes a solc for IDEs
        foundry = lib.makeOverridable
          ({ solc }: pkgs.symlinkJoin
            {
              name = "foundry";
              paths = with crateBuilds; [
                solc
                anvil
                cast
                chisel
                forge
              ];
            })
          { inherit (pkgs) solc; };
      in
      rec {
        packages = {
          default = foundry;
          inherit (crateBuilds) anvil cast chisel forge;
        };
        devShells.default = pkgs.mkShell {
          inputsFrom = lib.attrValues packages;
          packages = with pkgs; [
            pkgs.solc
            rust-analyzer-unwrapped
          ];

          # Environment variables
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };
      });
}
