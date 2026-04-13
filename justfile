alias e := export
alias r := refresh
alias u := update

modpacks_dir := "modpacks"
build_dir := "artifacts"

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --list

[private]
[windows]
_batchcmd cmd pack_id:
    Get-ChildItem -Path {{modpacks_dir}}\{{pack_id}} -Directory | % { \
      Set-Location $_.FullName; \
      Write-Host "Running '{{ cmd }}' in $($_.Name)"; \
      Invoke-Expression "{{ cmd }}"; \
      Pop-Location; \
    }

[linux]
[macos]
[private]
_batchcmd cmd pack_id:
    for d in {{modpacks_dir}}/{{pack_id}}/*/; do \
      [ -d "$d" ] || continue; \
      pushd "$d" > /dev/null; \
      echo "Running '{{ cmd }}' in $(basename "$d")..."; \
      {{ cmd }}; \
      popd > /dev/null; \
    done

# Export
[linux]
[macos]
export pack_id:
    mkdir -p {{build_dir}}
    @just _batchcmd "packwiz modrinth export --output ../../../{{build_dir}}/{{pack_id}}-{{`date +%s`}}.mrpack" {{pack_id}}

[windows]
export pack_id:
    if (!(Test-Path {{build_dir}})) { New-Item -ItemType Directory -Path {{build_dir}} }
    @just _batchcmd "packwiz modrinth export --output ../../../{{build_dir}}/{{pack_id}}.mrpack" {{pack_id}}

# Refresh
refresh pack_id:
    @just _batchcmd "packwiz refresh" {{pack_id}}

# Update
update pack_id:
    @just _batchcmd "packwiz update --all" {{pack_id}}

# Forked from the SkywardMC justfile, by devin aka intergrav, which is under the MIT license.