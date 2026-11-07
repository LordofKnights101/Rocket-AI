@echo off
title ROCKET AI Companion Launcher

echo Starting ROCKET AI Companion...
echo ==============================
echo.

cd /d "Z:\Coding\Ai Helper (ROCKET)"

echo Launching application...
start "" "target\release\rocket-ai.exe"

echo.
echo ROCKET AI Companion has been launched!
echo.
echo If you don't see the application, check your taskbar or task manager.
echo.
echo This window will close in 3 seconds...
timeout /t 3 /nobreak >nul
exit