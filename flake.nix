{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    naersk,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = (import nixpkgs) {inherit system overlays;};
      toolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src"];
      };
      nativeBuildInputs = with pkgs; [
        toolchain
      ];
      buildInputs = with pkgs; [];
      naersk' = pkgs.callPackage naersk {
        cargo = toolchain;
        rustc = toolchain;
        clippy = toolchain;
      };
    in let
      app = naersk'.buildPackage {
        src = ./.;
        nativeBuildInputs = nativeBuildInputs;
        buildInputs = buildInputs;
      };
    in {
      defaultPackage = app;
      packages = {
        container = pkgs.dockerTools.buildImage {
          name = "shabby";
          config = {
            entrypoint = ["${app}/bin/shabby"];
          };
        };
      };
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [] ++ nativeBuildInputs;
        buildInputs = with pkgs; [] ++ buildInputs;
      };
    });
}
