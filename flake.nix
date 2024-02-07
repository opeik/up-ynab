# Nix flake, see: https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
      in
        with pkgs; {
          # `nix fmt`
          formatter = alejandra;
          # `nix develop`
          devShells.default = mkShell {
            packages = [
              self.formatter.${system}
              _1password
              cargo-expand
              cargo-udeps
              darwin.apple_sdk.frameworks.SystemConfiguration
              (rust-bin.nightly.latest.default.override {
                extensions = ["rust-src"];
              })
            ];
          };
        }
    );
}
