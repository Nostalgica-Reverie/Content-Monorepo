{
  sha256,
  pkgs ? import <nixpkgs> {},
}:
pkgs.callPackage (import ./.) {
  vendorHash = sha256;
}
// {
  outputHash = sha256;
}
