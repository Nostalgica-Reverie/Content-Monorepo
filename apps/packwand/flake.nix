{
  description = "Packwand command-line interface";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs = {
    self,
    nixpkgs,
  }:
    let
      inherit (nixpkgs.lib)
        elem
        filter
        genAttrs
        substring
        ;

      # Packwand supports every 64-bit system on which Nixpkgs supports Go.
      explicitlyUnsupportedSystems = [ ];
      supportedSystems = filter (
        system: !(elem system explicitlyUnsupportedSystems)
      ) (import "${nixpkgs}/lib/systems/flake-systems.nix" { });
      forAllSystems = genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
        in
        rec {
          packwand = pkgs.callPackage ./nix {
            version = substring 0 8 (self.rev or "dirty");
            vendorHash = nixpkgs.lib.fileContents ./nix/vendor-hash;
          };
          default = packwand;
        }
      );

      checks = forAllSystems (system: {
        inherit (self.packages.${system}) packwand;
      });

      formatter = forAllSystems (system: nixpkgsFor.${system}.alejandra);
    };
}
