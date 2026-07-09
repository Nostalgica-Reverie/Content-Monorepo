let
  # Import nixpkgs if needed
  pkgs = import <nixpkgs> {};
in
  {
    lib ? pkgs.lib,
    buildGoModule ? pkgs.buildGoModule,
    fetchFromGitHub ? pkgs.fetchFromGitHub,
    installShellFiles ? pkgs.installShellFiles,
    # version and vendorHash should be specified by the caller
    version ? "latest",
    vendorHash,
  }:
    buildGoModule rec {
      pname = "packwand";
      inherit version vendorHash;

      src = ./..;

      nativeBuildInputs = [
        installShellFiles
      ];

      # Install shell completions
      postInstall = ''
        installShellCompletion --cmd packwand \
          --bash <($out/bin/packwand completion bash) \
          --fish <($out/bin/packwand completion fish) \
          --zsh <($out/bin/packwand completion zsh)
      '';

      meta = with lib; {
        description = "Minecraft modpack toolchain — packwiz core with multi-pack workspace management";
        license = licenses.mit;
        mainProgram = "packwand";
      };
    }
