#requires -Version 5.1
[CmdletBinding()]
param(
    [string]$Destination = (Join-Path $PSScriptRoot "..\crates\scout-convert\vendor\pdfium\x64"),
    [string]$Url = "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-win-x64.tgz"
)

$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$temp = Join-Path ([System.IO.Path]::GetTempPath()) ("pdfium-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $temp | Out-Null

try {
    $archive = Join-Path $temp "pdfium-win-x64.tgz"
    Write-Host "Downloading $Url"
    Invoke-WebRequest -Uri $Url -OutFile $archive -UseBasicParsing

    Write-Host "Extracting"
    & tar -xzf $archive -C $temp
    if ($LASTEXITCODE -ne 0) {
        throw "tar failed with exit code $LASTEXITCODE"
    }

    $dll = Get-ChildItem -Path $temp -Recurse -Filter "pdfium.dll" | Select-Object -First 1
    if (-not $dll) {
        throw "pdfium.dll was not found in the archive"
    }

    New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    $target = Join-Path $Destination "pdfium.dll"
    Copy-Item -LiteralPath $dll.FullName -Destination $target -Force

    $size = [math]::Round((Get-Item -LiteralPath $target).Length / 1MB, 1)
    Write-Host "Installed $target ($size MB)"
    Write-Host "Set SCOUT_PDFIUM_PATH to this file, or keep it next to scout-convert.exe."
}
finally {
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
