# Writes npm userconfig auth from repo-root `.env` (NPM_TOKEN). Token is not printed.
# Usage (from repo root):  pwsh -File scripts/sync-npm-userconfig-from-env.ps1
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$envFile = Join-Path $root ".env"
if (-not (Test-Path $envFile)) {
    Write-Error "Missing $envFile - copy .env.example to .env and set NPM_TOKEN."
    exit 1
}
$token = $null
Get-Content $envFile | ForEach-Object {
    $line = $_.Trim()
    if ($line -eq "" -or $line.StartsWith("#")) { return }
    $eq = $line.IndexOf("=")
    if ($eq -lt 1) { return }
    $key = $line.Substring(0, $eq).Trim()
    if ($key -ne "NPM_TOKEN") { return }
    $val = $line.Substring($eq + 1).Trim()
    if (($val.StartsWith('"') -and $val.EndsWith('"')) -or ($val.StartsWith("'") -and $val.EndsWith("'"))) {
        $val = $val.Substring(1, $val.Length - 2)
    }
    $script:token = $val
}
if ([string]::IsNullOrWhiteSpace($token)) {
    Write-Error "NPM_TOKEN is empty in $envFile."
    exit 1
}
& npm config set "//registry.npmjs.org/:_authToken" $token | Out-Null
Write-Host "npm userconfig updated for registry.npmjs.org (token not shown). Next: npm whoami"
