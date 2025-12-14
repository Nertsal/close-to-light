{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    geng.url = "github:geng-engine/cargo-geng/aa2c2242d7024734f3d5f6e5c9879a19c01b100a";
    geng.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { geng, nixpkgs, systems, self }:
    let
      pkgsFor = system: import nixpkgs { inherit system; };
      forEachSystem = f: nixpkgs.lib.genAttrs (import systems) (system:
        let
          pkgs = pkgsFor system;
        in
        f { inherit system pkgs; });
    in
    {
      devShells = forEachSystem ({ system, pkgs, ... }:
        {
          default = geng.lib.mkShell {
            inherit system;
            target.linux.enable = true;
            target.web.enable = true;
            packages = with pkgs; [
              just
              butler
              sqlite
              rlwrap
              sqlx-cli
            ];
          };
        });
      formatter = forEachSystem ({ pkgs, ... }: pkgs.nixfmt);
    };
}
