{
  description = "Lasting Legacy monorepo build and modpack tooling";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  };

  outputs = {
    self,
    nixpkgs,
    ...
  }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system: f system nixpkgs.legacyPackages.${system}
        );
      packwand2nixLib = import ./packages/packwand2nix/lib;
      mkPackwand =
        pkgs:
        pkgs.callPackage ./apps/packwand/nix {
          version = nixpkgs.lib.substring 0 8 (self.rev or "dirty");
          vendorHash = nixpkgs.lib.fileContents ./apps/packwand/nix/vendor-hash;
        };
      mkCursorapi =
        pkgs:
        pkgs.callPackage ./apps/api/nix {
          version = nixpkgs.lib.substring 0 8 (self.rev or "dirty");
          vendorHash = nixpkgs.lib.fileContents ./apps/api/nix/vendor-hash;
        };

      # Every modpack subdirectory with a generated checksums.json. Packwand
      # owns generation; Nix consumes the verified URLs and hashes without
      # re-parsing the pack indexes.
      packsWithChecksums =
        let
          inherit (builtins)
            attrNames
            concatMap
            filter
            pathExists
            readDir
            ;
          packs = filter (name: (readDir ./modpacks).${name} == "directory") (
            attrNames (readDir ./modpacks)
          );
          subdirsOf =
            pack:
            let
              contents = readDir (./modpacks + "/${pack}");
            in
            filter (
              sub:
              contents.${sub} == "directory"
              && pathExists (./modpacks + "/${pack}/${sub}/checksums.json")
            ) (attrNames contents);
        in
        concatMap (pack: map (sub: { inherit pack sub; }) (subdirsOf pack)) packs;

      mkPackwizPackages =
        pkgs:
        builtins.listToAttrs (
          map (entry: {
            name = "${entry.pack}/${entry.sub}";
            value = packwand2nixLib.mkPackwizPackages pkgs (
              ./modpacks + "/${entry.pack}/${entry.sub}/checksums.json"
            );
          }) packsWithChecksums
        );

      # Parse every generated checksum document during nix flake check without
      # downloading the potentially very large collection of mod JARs.
      modpackInventory = map (
        entry:
        let
          checksums = builtins.fromJSON (
            builtins.readFile (./modpacks + "/${entry.pack}/${entry.sub}/checksums.json")
          );
        in
        {
          name = "${entry.pack}/${entry.sub}";
          mods = builtins.attrNames checksums;
        }
      ) packsWithChecksums;
    in
    {
      # Re-export the packwiz2nix helpers for downstream server flakes.
      lib = packwand2nixLib;

      # legacyPackages.<system>."<pack>/<subdir>" is an attribute set of
      # fixed-output mod derivations suitable for mkModLinks.
      legacyPackages = forAllSystems (_system: pkgs: mkPackwizPackages pkgs);

      packages = forAllSystems (_system: pkgs: rec {
        cursorapi = mkCursorapi pkgs;
        packwand = mkPackwand pkgs;
        default = packwand;
      });

      checks = forAllSystems (_system: pkgs: {
        cursorapi = mkCursorapi pkgs;
        packwand = mkPackwand pkgs;
        modpack-inventory = pkgs.writeText "lasting-legacy-modpack-inventory.json" (
          builtins.toJSON modpackInventory
        );
      });

      apps = forAllSystems (system: _pkgs: rec {
        cursorapi = {
          type = "app";
          program = nixpkgs.lib.getExe self.packages.${system}.cursorapi;
        };
        packwand = {
          type = "app";
          program = nixpkgs.lib.getExe self.packages.${system}.packwand;
        };
        default = packwand;
      });

      devShells = forAllSystems (_system: pkgs: {
        default = pkgs.mkShellNoCC {
          packages = [
            pkgs.alejandra
            pkgs.go_1_26
            pkgs.just
          ];
        };
      });

      formatter = forAllSystems (_system: pkgs: pkgs.alejandra);
    };
}
