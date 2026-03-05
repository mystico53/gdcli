$ErrorActionPreference = "Stop"

$repo = "mystico53/gdcli"
$installDir = "$env:LOCALAPPDATA\gdcli"
$archive = "gdcli-windows-x86_64.zip"

# Get latest release tag
Write-Host "Fetching latest release..."
$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
$tag = $release.tag_name

Write-Host "Installing gdcli $tag..."

# Download zip
$url = "https://github.com/$repo/releases/download/$tag/$archive"
$zipPath = "$env:TEMP\gdcli.zip"
Invoke-WebRequest -Uri $url -OutFile $zipPath

# Extract
if (Test-Path $installDir) {
    Remove-Item -Recurse -Force $installDir
}
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Expand-Archive -Path $zipPath -DestinationPath $installDir -Force
Remove-Item $zipPath

Write-Host "Installed gdcli to $installDir"

# Add to user PATH if not already present
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$installDir;$userPath", "User")
    Write-Host "Added $installDir to user PATH. Restart your terminal to use gdcli."
} else {
    Write-Host "gdcli is already in PATH."
}
