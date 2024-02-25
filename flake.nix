{
  description = "Development";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        waylandDeps = with pkgs; [
          libxkbcommon
          wayland
        ];
        xorgDeps = with pkgs; [
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
        libDeps = with pkgs; waylandDeps ++ xorgDeps ++ [
            alsa-lib
            udev
            libGL
            xorg.libxcb
            cmake
            fontconfig
            mesa
            freeglut
        ];
        libPath = pkgs.lib.makeLibraryPath libDeps;
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = libDeps ++ [
            sqlite
            rlwrap
            gcc
            openssl
            pkg-config
            # rust-bin.stable.latest.default
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
          ];
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${libPath}"
            export WINIT_UNIX_BACKEND=x11
          '';
        };
      }
    );
}
