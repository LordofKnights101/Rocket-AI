@echo off
echo Creating Start Menu shortcut for ROCKET AI Companion...
echo.

:: Create Start Menu shortcut
set startmenu=%AppData%\Microsoft\Windows\Start Menu\Programs\ROCKET AI Companion.lnk

powershell -Command "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('%startmenu%'); $s.TargetPath = 'Z:\Coding\Ai Helper (ROCKET)\target\release\rocket-ai.exe'; $s.WorkingDirectory = 'Z:\Coding\Ai Helper (ROCKET)'; $s.Description = 'ROCKET AI Companion - Your Personal Gaming AI'; $s.Save()"

echo Start Menu shortcut created!
echo.
echo You can now find ROCKET in your Start Menu under Programs.
echo.
echo You can also use these one-click launchers:
echo • Double-click ROCKET.bat to launch immediately
echo • Double-click setup_rocket.bat for more options
echo.
echo Setup complete!