param (
    [Parameter(Mandatory = $true)] [string]$Version,
    [Parameter(Mandatory = $true)] [string]$OutputDir,
    [Parameter(Mandatory = $false)] [string]$ReleaseDate
)

Write-Host "--------------------------------------------------"
Write-Host "[START] Generating Update Notification Data"
Write-Host "Version: $Version"
Write-Host "Output Dir: $OutputDir"
Write-Host "--------------------------------------------------"

# 1. Create Output Directory structure
$ApiPath = Join-Path $OutputDir "common/v1/releases"
if (-not (Test-Path $ApiPath)) {
    New-Item -ItemType Directory -Path $ApiPath -Force | Out-Null
    Write-Host "[OK] Created directory: $ApiPath"
}

# 2. Generate JSON Content
# Use provided ReleaseDate or fall back to current UTC+8 date
if ([string]::IsNullOrWhiteSpace($ReleaseDate)) {
    $ReleaseDate = (Get-Date).ToUniversalTime().AddHours(8).ToString("yyyy-MM-dd")
    Write-Host "[INFO] No release_date provided, using current date: $ReleaseDate"
}

# Use ordered hashtable to keep the exact order if desired, though JSON doesn't strictly require it
$JsonData = [ordered]@{
    err  = 0
    msg  = "success"
    data = [ordered]@{
        version      = $Version
        release_date = $ReleaseDate
        download_url = "https://www.wslui.com/download/"
    }
}

# Convert to JSON with -Compress to make it a single line as requested
$JsonString = $JsonData | ConvertTo-Json -Depth 10 -Compress

# Write to file without extension (force UTF-8 without BOM using .NET API)
$LatestFilePath = Join-Path $ApiPath "latest"
$Utf8NoBom = [System.Text.UTF8Encoding]::new($false)
[System.IO.File]::WriteAllText($LatestFilePath, $JsonString, $Utf8NoBom)
Write-Host "[OK] Generated JSON at: $LatestFilePath"
Write-Host "[INFO] JSON Content: $JsonString"

# 3. Generate _headers file
$HeadersPath = Join-Path $OutputDir "_headers"
$HeadersContent = @"
/common/*
  Content-Type: application/json; charset=utf-8
  Cache-Control: public, s-maxage=60, max-age=0
"@

[System.IO.File]::WriteAllText($HeadersPath, $HeadersContent, $Utf8NoBom)
Write-Host "[OK] Generated _headers file at: $HeadersPath"

Write-Host "`n--------------------------------------------------"
Write-Host "[SUCCESS] Update Notification Preparation Complete"
Write-Host "--------------------------------------------------`n"
exit 0
