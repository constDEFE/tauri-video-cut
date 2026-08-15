# scripts/package-release.ps1
# Stages distributable files and creates a release ZIP.

param(
    [string]$ExeName   = "tauri-video-cut.exe",
    [string]$TargetDir = "src-tauri/target/release",
    [string]$OutDir    = "dist-release"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path "$TargetDir/$ExeName")) {
    Write-Error "Executable not found: $TargetDir/$ExeName. Run 'bun run tauri build' first."
    exit 1
}
if (-not (Test-Path "$TargetDir/lib")) {
    Write-Error "lib/ directory not found in $TargetDir. Run harvest + build first."
    exit 1
}

if (Test-Path $OutDir) { Remove-Item -Recurse -Force $OutDir }
New-Item -ItemType Directory -Force -Path "$OutDir/lib" | Out-Null

Write-Host "Staging executable..."
Copy-Item "$TargetDir/$ExeName" "$OutDir/"

Write-Host "Staging bundled libraries (ffmpeg, ffprobe, DLLs)..."
Copy-Item "$TargetDir/lib/*" "$OutDir/lib/" -Recurse

Write-Host "Staging legal documents from repository root..."

New-Item -ItemType Directory -Force -Path "$OutDir/legal" | Out-Null

Copy-Item "LICENSE"              "$OutDir/legal/"
Copy-Item "LICENSE.DISTRIBUTION" "$OutDir/legal/"
Copy-Item "NOTICE.md"            "$OutDir/legal/"
Copy-Item "SOURCE-OFFER.md"      "$OutDir/legal/"

if (Test-Path "LICENSES") {
    Copy-Item "LICENSES" "$OutDir/legal/LICENSE TEXTS" -Recurse
}

New-Item -ItemType Directory -Force -Path "$OutDir/legal/generated/msys2" | Out-Null

$genFiles = @(
    "THIRD_PARTY_LICENSES.md"
    "frontend-licenses.md"
    "rust-licenses.md"
    "rust-licenses.html"
    "build-env.txt"
    "libmpv-wrapper-LICENSE"
)
foreach ($f in $genFiles) {
    if (Test-Path "legal/generated/$f") {
        Copy-Item "legal/generated/$f" "$OutDir/legal/generated/$f"
    }
}

foreach ($d in @("ffmpeg", "mpv")) {
    if (Test-Path "legal/generated/$d") {
        Copy-Item "legal/generated/$d" "$OutDir/legal/generated/$d" -Recurse
    }
}

# MSYS2 handled separately to avoid double-nesting
if (Test-Path "legal/generated/msys2") {
    Copy-Item "legal/generated/msys2/*" "$OutDir/legal/generated/msys2/" -Recurse -Force
}

$zip = "windows-x64.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Write-Host "Compressing to $zip ..."
Compress-Archive -Path "$OutDir/*" -DestinationPath $zip

Write-Host ""
Write-Host "SUCCESS: Created $zip"
Write-Host "Contents:"
Get-ChildItem "$OutDir" -Recurse -File |
    ForEach-Object { $_.FullName.Replace((Resolve-Path $OutDir).Path + "\", "") } |
    Sort-Object
