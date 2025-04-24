# vim:ts=2:sw=2:et:
{
  fenixToolchain,
  lib,
  makeRustPlatform,
}:
(makeRustPlatform {
  cargo = fenixToolchain;
  rustc = fenixToolchain;
})
.buildRustPackage {
  name = "statusline";

  src = lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;

  # FIXME(25.05): edition 2024 is not supported yet
  auditable = false;
}
