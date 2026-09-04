{
  description = "Bounded vector development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-darwin"
      ];
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rust-bin.stable.latest.default
              cargo-hack
            ];
          };
        }
      );

      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          check = pkgs.writeShellApplication {
            name = "check";
            runtimeInputs = [
              pkgs.rust-bin.stable.latest.default
              pkgs.cargo-hack
            ];
            text = ''
              cargo hack check --feature-powerset --no-dev-deps --exclude-features=nightly
              cargo hack test --each-feature --exclude-features=nightly
              cargo hack clippy --each-feature --exclude-features=nightly
              cargo hack check --feature-powerset --no-dev-deps
              cargo hack test --each-feature
              cargo hack clippy --each-feature
              cargo fmt --all -- --check --color always
            '';
          };
        }
      );
    };
}
