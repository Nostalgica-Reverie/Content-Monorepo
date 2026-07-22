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

$retainedExtensions = @(
    'bat',
    'configuration-editing',
    'cpp',
    'css',
    'css-language-features',
    'diff',
    'dotenv',
    'git',
    'git-base',
    'hlsl',
    'html',
    'html-language-features',
    'ini',
    'java',
    'javascript',
    'json',
    'json-language-features',
    'log',
    'markdown-basics',
    'markdown-language-features',
    'markdown-math',
    'media-preview',
    'merge-conflict',
    'packwand',
    'search-result',
    'shaderlab',
    'shellscript',
    'theme-defaults',
    'theme-seti',
    'types',
    'typescript-basics',
    'typescript-language-features',
    'xml',
    'yaml'
)

$extensionsRoot = Join-Path $root 'extensions'
$removed = @()
Get-ChildItem -LiteralPath $extensionsRoot -Directory | ForEach-Object {
    if ($_.Name -notin $retainedExtensions) {
        $target = [IO.Path]::GetFullPath($_.FullName)
        $prefix = $extensionsRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
        if (-not $target.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw ('Refusing to remove extension outside the fork: ' + $target)
        }
        Remove-Item -Recurse -Force -LiteralPath $target
        $removed += $_.Name
    }
}

Write-Output ('Retained {0} Packwand-relevant extensions and removed {1}: {2}' -f $retainedExtensions.Count, $removed.Count, ($removed -join ', '))
