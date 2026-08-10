{
  buildGo126Module,
  lib,
  vendorHash,
  version ? "latest",
}:
buildGo126Module {
  pname = "packwand-social";
  inherit vendorHash version;

  src = lib.fileset.toSource {
    root = ../../../..;
    fileset = ../../../.. + "/apps/packwandrs/packwand-social";
  };
  modRoot = "apps/packwandrs/packwand-social";
  subPackages = [ "." ];

  meta = {
    description = "Local ATProto identity bridge for Packwand";
    license = lib.licenses.agpl3Plus;
    mainProgram = "packwand-social";
    platforms = lib.platforms.unix;
  };
}
