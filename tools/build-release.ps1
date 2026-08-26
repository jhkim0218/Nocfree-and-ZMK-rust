param(
    [ValidateSet('ANSI', 'ISO', 'JIS', 'KR')]
    [string]$Layout = 'ANSI'
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$python = Get-Command python -ErrorAction Stop
& $python.Source -B (Join-Path $PSScriptRoot 'build_release.py') --layout $Layout
if ($LASTEXITCODE -ne 0) {
    throw ('Portable release build failed (exit {0})' -f $LASTEXITCODE)
}
