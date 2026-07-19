{
  buildGo126Module,
  installShellFiles,
  lib,
  vendorHash,
  version ? "latest",
}:
buildGo126Module {
  pname = "packwand";
  inherit vendorHash version;

  src = lib.cleanSource ./..;
  subPackages = [ "." ];

  nativeBuildInputs = [ installShellFiles ];

  postInstall = ''
    installShellCompletion --cmd packwand \
      --bash <($out/bin/packwand completion bash) \
      --fish <($out/bin/packwand completion fish) \
      --zsh <($out/bin/packwand completion zsh)
  '';

  meta = {
    description = "Minecraft modpack toolchain with multi-pack workspace management";
    license = lib.licenses.mit;
    mainProgram = "packwand";
    platforms = lib.platforms.unix;
  };
}
