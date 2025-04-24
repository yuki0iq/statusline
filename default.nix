# vim:ts=2:sw=2:et:
let
  pins = import ./npins;
  pkgs = import pins.nixpkgs {};
in
  pkgs.callPackage ./statusline.nix {
    fenixToolchain = (pkgs.callPackage pins.fenix {}).minimal.toolchain;
  }
