param (
    [Parameter(Mandatory=$true)] [string]$Version
)

Write-Host "--------------------------------------------------"
Write-Host "[START] Preparing assets for version: $Version"
Write-Host "--------------------------------------------------"

# 1. Call Inno Setup to build the installer
Write-Host "[INFO] Building Installer using Inno Setup..."
try {
    .\build\setup\build.ps1 -Version "$Version" -ErrorAction Stop
    Write-Host "[OK] Installer build completed."
} catch {
    Write-Error "[ERROR] Failed to build installer: $_"
    exit 1
}

# 2. Prepare Portable Assets
Write-Host "`n[INFO] Preparing Portable assets..."
$release_dir = "target/release"
$raw_exe = "wsldashboard.exe"
$portable_exe = "WSLDashboard.$Version.Portable.x64.exe"
$portable_zip = "WSLDashboard.$Version.Portable.x64.zip"

if (Test-Path "$release_dir/$raw_exe") {
    Push-Location $release_dir
    try {
        # Rename to versioned name
        Move-Item "$raw_exe" "$portable_exe" -Force
        Write-Host "[OK] Renamed raw exe to $portable_exe"
        
        # Use 7z for maximum compression
        Write-Host "[INFO] Compressing Portable ZIP (7z mx9)..."
        7z a -tzip -mx9 "$portable_zip" "$portable_exe" | Out-Null
        Write-Host "[OK] Created: $portable_zip"
    } finally {
        Pop-Location
    }
} else {
    Write-Error "[ERROR] Portable exe not found at $release_dir/$raw_exe"
    exit 1
}

# 3. Prepare Setup ZIP
Write-Host "`n[INFO] Preparing Setup ZIP..."
$setup_dir = "build/releases"
$setup_exe = "WSLDashboard.$Version.Setup.x64.exe"
$setup_zip = "WSLDashboard.$Version.Setup.x64.zip"

if (Test-Path "$setup_dir/$setup_exe") {
    Push-Location $setup_dir
    try {
        Write-Host "[INFO] Compressing Setup ZIP (7z mx9)..."
        7z a -tzip -mx9 "$setup_zip" "$setup_exe" | Out-Null
        Write-Host "[OK] Created: $setup_zip"
    } finally {
        Pop-Location
    }
} else {
    Write-Error "[ERROR] Setup exe not found at $setup_dir/$setup_exe"
    exit 1
}

Write-Host "`n[SUCCESS] All assets are prepared and compressed."
Write-Host "--------------------------------------------------`n"

exit 0
