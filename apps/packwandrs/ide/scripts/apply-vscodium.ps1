param(
    [Parameter(Mandatory = $true)]
    [string]$VscodeRoot,
    [Parameter(Mandatory = $true)]
    [string]$VscodiumRoot,
    [ValidateSet('common', 'linux', 'osx', 'windows')]
    [string]$Platform = 'common'
)

$ErrorActionPreference = 'Stop'
$resolvedVscodeRoot = (Resolve-Path -LiteralPath $VscodeRoot).Path
$resolvedVscodiumRoot = (Resolve-Path -LiteralPath $VscodiumRoot).Path
$upstream = Get-Content -Raw (Join-Path $resolvedVscodiumRoot 'upstream/stable.json') | ConvertFrom-Json
$actualCommit = (& git -C $resolvedVscodeRoot rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $actualCommit -ne $upstream.commit) {
    throw ('VS Code must be checked out at VSCodium stable commit {0}; found {1}' -f $upstream.commit, $actualCommit)
}

$replacement = [ordered]@{
    '!!APP_NAME!!' = 'Packwand IDE'
    '!!APP_NAME_LC!!' = 'packwand ide'
    '!!ASSETS_REPOSITORY!!' = 'Lasting-Legacy/Lasting-Legacy-Monorepo'
    '!!BINARY_NAME!!' = 'packwand-ide'
    '!!GH_REPO_PATH!!' = 'Lasting-Legacy/Lasting-Legacy-Monorepo'
    '!!GLOBAL_DIRNAME!!' = 'packwand'
    '!!ORG_NAME!!' = 'Lasting Legacy'
    '!!RELEASE_VERSION!!' = '0.1.0'
    '!!TUNNEL_APP_NAME!!' = 'packwand-ide-tunnel'
}

$patchDirectories = @((Join-Path $resolvedVscodiumRoot 'patches'))
if ($Platform -ne 'common') {
    $patchDirectories += Join-Path $resolvedVscodiumRoot ('patches/' + $Platform)
}

foreach ($patchDirectory in $patchDirectories) {
    Get-ChildItem -LiteralPath $patchDirectory -Filter '*.json' -File | Sort-Object Name | ForEach-Object {
        $actions = Get-Content -Raw -LiteralPath $_.FullName | ConvertFrom-Json
        foreach ($action in $actions) {
            if ($action.action -ne 'remove') { throw ('Unsupported VSCodium action: ' + $action.action) }
            foreach ($relativePath in $action.paths) {
                $target = [IO.Path]::GetFullPath((Join-Path $resolvedVscodeRoot $relativePath))
                $rootPrefix = $resolvedVscodeRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
                if (-not $target.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                    throw ('Refusing to remove path outside VS Code: ' + $target)
                }
                if (Test-Path -LiteralPath $target) { Remove-Item -Recurse -Force -LiteralPath $target }
            }
        }
    }
}

$workingDirectory = Join-Path ([IO.Path]::GetTempPath()) ('packwand-vscodium-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $workingDirectory | Out-Null
try {
    foreach ($patchDirectory in $patchDirectories) {
        Get-ChildItem -LiteralPath $patchDirectory -Filter '*.patch' -File | Sort-Object Name | ForEach-Object {
            $content = [IO.File]::ReadAllText($_.FullName)
            foreach ($entry in $replacement.GetEnumerator()) { $content = $content.Replace($entry.Key, $entry.Value) }
            $preparedPatch = Join-Path $workingDirectory $_.Name
            [IO.File]::WriteAllText($preparedPatch, $content, [Text.UTF8Encoding]::new($false))
            & git -C $resolvedVscodeRoot apply --ignore-whitespace $preparedPatch
            if ($LASTEXITCODE -ne 0) { throw ('Failed to apply VSCodium patch ' + $_.Name) }
        }
    }
} finally {
    Remove-Item -Recurse -Force -LiteralPath $workingDirectory
}

Write-Output ('Applied VSCodium patches for {0} to VS Code {1} ({2}).' -f $Platform, $upstream.tag, $actualCommit)
