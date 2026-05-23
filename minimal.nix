# use
# ```
# nix develop minimal.nix --ignore-env --impure --keep DISPLAY --keep TERM --keep NIX_SSL_CERT_FILE --keep XDG_RUNTIME_DIR
# ```
# to test the library dependencies

{
  description = "A nothing flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        libDeps = with pkgs; [
        ];
        libPath = pkgs.lib.makeLibraryPath libDeps;
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = libDeps ++ [
            steam-run
          ];
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${libPath}"
            export WINIT_UNIX_BACKEND=x11
          '';
        };
      }
    );
}

