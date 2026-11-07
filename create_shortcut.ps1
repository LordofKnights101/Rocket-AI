# ROCKET AI Companion Desktop Shortcut Creator
# Run this script to create a desktop shortcut for easy access

$desktopPath = [System.Environment]::GetFolderPath('Desktop')
$shortcutPath = Join-Path $desktopPath "ROCKET AI Companion.lnk"
$targetPath = "Z:\Coding\Ai Helper (ROCKET)\launch_rocket.bat"

# Create shortcut using Windows Script Host
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $targetPath
$shortcut.WorkingDirectory = "Z:\Coding\Ai Helper (ROCKET)"
$shortcut.Description = "ROCKET AI Companion - Your Personal Gaming AI"
$shortcut.IconLocation = "Z:\Coding\Ai Helper (ROCKET)\target\release\rocket-ai.exe, 0"
$shortcut.Save()

Write-Host "ROCKET AI Companion shortcut created on your desktop!"
Write-Host "Double-click the shortcut to launch the application."