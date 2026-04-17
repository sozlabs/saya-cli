# Load KEY=value lines from repo-root `.env` into the current process (secrets stay off stdout).
# Usage (from repo root, same shell):  . ./scripts/load-env.ps1
# One-shot:  pwsh -NoProfile -Command "& { . ./scripts/load-env.ps1; npm whoami }"
$ErrorActionPreference = "Stop"
# Repo root = parent of `scripts/` (saya-cli/)
$root = Split-Path -Parent $PSScriptRoot
$envFile = Join-Path $root ".env"
if (-not (Test-Path $envFile)) {
    Write-Error "Missing $envFile - copy .env.example to .env and set NPM_TOKEN."
    exit 1
}
Get-Content $envFile | ForEach-Object {
    $line = $_.Trim()
    if ($line -eq "" -or $line.StartsWith("#")) { return }
    $eq = $line.IndexOf("=")
    if ($eq -lt 1) { return }
    $key = $line.Substring(0, $eq).Trim()
    $val = $line.Substring($eq + 1).Trim()
    if (($val.StartsWith('"') -and $val.EndsWith('"')) -or ($val.StartsWith("'") -and $val.EndsWith("'"))) {
        $val = $val.Substring(1, $val.Length - 2)
    }
    [Environment]::SetEnvironmentVariable($key, $val, "Process")
    if ($key -eq "NPM_TOKEN" -and -not [string]::IsNullOrWhiteSpace($val)) {
        [Environment]::SetEnvironmentVariable("NODE_AUTH_TOKEN", $val, "Process")
    }
}
Write-Host "Loaded .env into process environment (values not printed)."
