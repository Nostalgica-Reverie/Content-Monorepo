{
  buildGo126Module,
  lib,
  vendorHash,
  version ? "latest",
}:
buildGo126Module {
  pname = "somnus";
  inherit vendorHash version;

  src = lib.fileset.toSource {
    root = ../../../..;
    fileset = lib.fileset.unions [
      (../../../.. + "/apps/packwandrs/somnus")
      (../../../.. + "/.tangled/workflows")
    ];
  };
  modRoot = "apps/packwandrs/somnus";
  subPackages = [ "." ];

  meta = {
    description = "Local Tangled workflow runner for Packwand";
    license = lib.licenses.agpl3Plus;
    mainProgram = "somnus";
    platforms = lib.platforms.unix;
  };
}
