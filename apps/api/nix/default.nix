{
  buildGo126Module,
  lib,
  vendorHash,
  version ? "latest",
}:
buildGo126Module {
  pname = "cursorapi";
  inherit vendorHash version;

  src = lib.fileset.toSource {
    root = ../../..;
    fileset = lib.fileset.unions [
      ../../../apps/api
      ../../../apps/packwand
    ];
  };
  modRoot = "apps/api";
  subPackages = [ "./cursorapi" ];

  meta = {
    description = "Standalone host for Packwand's versioned manifest API";
    license = lib.licenses.mit;
    mainProgram = "cursorapi";
    platforms = lib.platforms.unix;
  };
}
