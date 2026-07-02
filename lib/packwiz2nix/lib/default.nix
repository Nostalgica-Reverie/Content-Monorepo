let
  inherit
    (builtins)
    attrNames
    concatStringsSep
    hashFile
    fetchurl
    fromJSON
    listToAttrs
    mapAttrs
    readDir
    readFile
    replaceStrings
    toFile
    toJSON
    ;

  # loads data from a toml json given
  # the directory (dir) and filename (name)
  # string -> string -> attrset
  fromMod = dir: name: fromTOML (readFile "${dir}/${name}");

  # replaces `.pw.toml` extensions with `.jar`
  # to correct the store paths of jarfiles
  # string -> string
  fixupName = replaceStrings [".pw.toml"] [".jar"];

  # *pkgs*.fetchurl wrapper that downloads a
  # jarfile mod. pkgs.fetchurl is used over builtins
  # here since we have a checksum and can take advantage
  # of fixed output derivations
  # attrset -> string -> attrset -> store path
  mkMod = pkgs: name: mod:
    pkgs.fetchurl {
      name = fixupName name;
      inherit (mod) url sha256;
    };

  # maps each mod in our checksums.json format
  # to the store path of a fixed output derivation
  # attrset -> attrset
  genMods = pkgs:
    mapAttrs (mkMod pkgs);

  # this is probably what you're looking for if
  # you're a developer trying to use this in your modpack.
  # this is where you create a checksums file for end users
  # to put into mkPackwizPackages, so make sure you keep it up to
  # date!
  #
  # `dir` is a path to the folder containing your .pw.toml files
  # files for mods. make sure they are the only files in the folder
  #
  # path -> file
  mkChecksums = dir: let
    mods = readDir dir;

    getChecksum = name: url:
      hashFile "sha256" (fetchurl {
        name = fixupName name;
        inherit url;
      });

    toWrite =
      toJSON
      (mapAttrs (mod: _: let
          data = fromMod dir mod;
        in {
          inherit (data.download) url;
          sha256 = getChecksum mod data.download.url;
        })
        mods);
  in
    toFile "checksums-json" toWrite;

  # this is probably what you're looking for if
  # you're an end user trying to implement a modpack in
  # your module.
  #
  # `pkgs` is an instance of nixpkgs for your system,
  # must at least contain `fetchurl`.
  #
  # `checksums` is a json file from an upstream modpack
  # containing the names, urls, and sha256sums of mods
  #
  # attrset -> path -> attrset
  mkPackwizPackages = pkgs: checksums: genMods pkgs (fromJSON (readFile checksums));
in {
  inherit mkChecksums mkPackwizPackages;

  # this another function modpack developers will be looking
  # for. it allows you to build a modpack compatible with MultiMC
  # and Prism Launcher, containing external files.
  #
  # `pkgs` is an instance of nixpkgs with at least lib, runCommand,
  # and zip
  #
  # `mods` is the result of mkPackwizPackages
  #
  # `filesDir` is the root path of all external files you want
  # included in your zipped modpack
  #
  # `name` is a string containing the name of your modpack
  mkMultiMCPack = {
    pkgs,
    mods,
    filesDir ? "",
    name,
  }: let
    inherit (pkgs) fetchurl lib runCommand zip;
    inherit (lib) forEach;

    packwiz-installer-bootstrap = fetchurl {
      url = "https://github.com/packwiz/packwiz-installer-bootstrap/releases/download/v0.0.3/packwiz-installer-bootstrap.jar";
      sha256 = "sha256-qPuyTcYEJ46X9GiOgtPZGjGLmO/AjV2/y8vKtkQ9EWw=";
    };

    forNames = attrset: forEach (attrNames attrset);
    commandSep = concatStringsSep "; ";
    modFiles = let
      commands = forNames mods (mod: "cp -r ${mods.${mod}} $out/decompressed/.minecraft/mods/");
    in
      commandSep commands;

    regularFiles =
      if filesDir != ""
      then
        (let
          files = readDir filesDir;
          commands = forNames files (file: "cp -r ${filesDir}/${file} $out/decompressed/");
        in
          commandSep commands)
      else "echo 'not copying external files...'";
  in
    runCommand name {} ''
      mkdir -p $out/decompressed/.minecraft/mods
      ${regularFiles}
      ${modFiles}
      cp ${packwiz-installer-bootstrap} $out/decompressed/.minecraft/packwiz-installer-bootstrap.jar

      cd $out/decompressed
      ${zip}/bin/zip -r ../${name}.zip {*,.*}
    '';

  # this creates an `apps` attribute for a flake
  # which runs a bash script to generate a checksums
  # file
  #
  # `pkgs` is an instance of nixpkgs for your system,
  # must at least contain `writeShellScriptBin`
  #
  # `dir` is a path to the folder containing your .pw.toml files
  # files for mods. make sure they are the only files in the folder
  #
  # attrset -> path -> attrset
  mkChecksumsApp = pkgs: dir: let
    inherit (pkgs) writeShellScriptBin;
    checksums = mkChecksums dir;
    name = "generate-checksums";
    script = writeShellScriptBin name ''
      cat ${checksums} > checksums.json
    '';
  in {
    type = "app";
    program = script.outPath + "/bin/${name}";
  };

  # this creates an attrset value for
  # minecraft-servers.servers.<server>.symlinks
  # attrset -> attrset
  mkModLinks = mods: let
    fixup = map (name: {
      name = "mods/" + fixupName name;
      value = mods.${name};
    }) (attrNames mods);
  in
    listToAttrs fixup;
}
