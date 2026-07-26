param(
    [Parameter(Mandatory = $true)]
    [string]$WorkbenchRoot
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkbenchRoot).Path
$package = Get-Content -Raw -LiteralPath (Join-Path $root 'package.json') | ConvertFrom-Json
if ($package.name -ne 'packwand-ide' -or $package.version -ne '1.126.0') {
    throw 'Refusing to prune a workbench that is not the pinned Packwand IDE fork'
}

$rootPrefix = $root.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar

# The web-only distribution never loads an Electron process, so the whole desktop
# runtime and packaging surface is dead weight. Removing it makes the
# `electron-runtime` entry in upstream.yml's removed_surfaces an enforced claim.
function Remove-Confined {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return $false
    }
    $target = [IO.Path]::GetFullPath($Path)
    if (-not $target.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw ('Refusing to remove path outside the fork: ' + $target)
    }
    Remove-Item -Recurse -Force -LiteralPath $target
    return $true
}

# Code OSS names its process layers by directory, so a basename sweep survives
# upstream resyncs without a hand-maintained path list.
$electronLayers = @('electron-browser', 'electron-main', 'electron-utility')

# Collect before deleting so the recursive walk never descends into a directory
# that has already been removed.
$layerDirectories = @(
    Get-ChildItem -LiteralPath (Join-Path $root 'src\vs') -Recurse -Directory |
        Where-Object { $_.Name -in $electronLayers } |
        Select-Object -ExpandProperty FullName
)

$removedLayers = 0
foreach ($directory in $layerDirectories) {
    if (Remove-Confined $directory) {
        $removedLayers++
    }
}

# Paths that do not follow the layer-directory convention.
$explicitPaths = @(
    # Electron main-process bootstrap.
    'src\main.ts',
    # Desktop entry points; every import below them is an electron-browser module.
    'src\vs\workbench\workbench.desktop.main.ts',
    'src\vs\sessions\sessions.desktop.main.ts',
    # Upstream release audit that side-effect-imports the desktop entry point to
    # enumerate every registered colour; the web distribution is not its subject.
    'src\vs\workbench\contrib\themes\test\node\colorRegistry.releaseTest.ts',
    # Orphaned Electron dev-tool mini-app.
    'build\builtin',
    # Gulpfiles are require()d wholesale by build/gulpfile.ts, so these must go
    # together with the @vscode/gulp-electron devDependency they import.
    'build\gulpfile.vscode.ts',
    'build\gulpfile.scan.ts',
    'build\gulpfile.vscode.win32.ts',
    'build\gulpfile.vscode.linux.ts',
    'build\lib\electron.ts',
    'build\lib\typings\@vscode\gulp-electron.d.ts',
    'build\checker\tsconfig.electron-browser.json',
    'build\checker\tsconfig.electron-main.json',
    'build\checker\tsconfig.electron-utility.json'
)

$removedPaths = @()
foreach ($relative in $explicitPaths) {
    if (Remove-Confined (Join-Path $root $relative)) {
        $removedPaths += $relative
    }
}

Write-Output ('Removed {0} Electron layer directories and {1} of {2} explicit Electron paths' -f $removedLayers, $removedPaths.Count, $explicitPaths.Count)
