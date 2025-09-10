{
  system ? builtins.currentSystem,
  pkgs ? import <nixpkgs> {
    inherit system;
  }
}:
pkgs.rustPlatform.buildRustPackage {
  name = "statusline";
  src = pkgs.lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
