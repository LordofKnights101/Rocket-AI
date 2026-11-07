@echo off
echo ========================================
echo  ROCKET AI COMPANION - SETUP COMPLETE!
echo ========================================
echo.
echo Your ROCKET AI Companion is ready to use!
echo.
echo 🚀 QUICK LAUNCH OPTIONS:
echo.
echo 1. CREATE DESKTOP SHORTCUT:
echo    Right-click create_shortcut.ps1 and "Run with PowerShell"
echo.
echo 2. USE BATCH FILE:
echo    Double-click launch_rocket.bat
echo.
echo 3. RUN DIRECTLY:
echo    Double-click target\release\rocket-ai.exe
echo.
echo 📁 APPLICATION LOCATION:
echo    Z:\Coding\Ai Helper (ROCKET)
echo.
echo 📋 FEATURES:
echo    • Game-specific cheat sheets (Minecraft, PUBG, Valorant, etc.)
echo    • Animated rocket mascot with flame effects
echo    • Local & Online AI modes
echo    • Multi-game support with smart responses
echo.
echo 🎮 GETTING STARTED:
echo    1. Launch the application
echo    2. Select your game from the dropdown
echo    3. Start chatting for gaming assistance
echo    4. Try "cheat sheet" for game guides
echo.
echo 🔧 FILES CREATED:
echo    • launch_rocket.bat    - Windows launcher
echo    • launch_rocket.sh     - Linux/macOS launcher  
echo    • create_shortcut.ps1  - Desktop shortcut creator
echo    • QUICKSTART.md       - Complete guide
echo    • target\release\rocket-ai.exe - Main application
echo.
echo ========================================
echo.
echo Choose your launch method:
echo.
echo [1] Create desktop shortcut (recommended)
echo [2] Launch directly now
echo [3] View quick start guide
echo [4] Exit
echo.

set /p choice="Enter your choice (1-4): "

if "%choice%"=="1" (
    echo Creating desktop shortcut...
    powershell -ExecutionPolicy Bypass -File create_shortcut.ps1
    echo.
    echo Shortcut created! Double-click it to launch ROCKET.
) else if "%choice%"=="2" (
    echo Launching ROCKET AI Companion...
    start "ROCKET AI Companion" target\release\rocket-ai.exe
    echo.
    echo ROCKET is launching! Check your taskbar.
) else if "%choice%"=="3" (
    echo.
    echo ========================================
    echo           QUICK START GUIDE
    echo ========================================
    type QUICKSTART.md
) else if "%choice%"=="4" (
    echo Goodbye! Launch ROCKET anytime using the scripts.
    pause
    exit /b 0
) else (
    echo Invalid choice. Please run this script again.
    pause
    exit /b 1
)

echo.
echo Setup complete! Enjoy your ROCKET AI Companion! 🚀
pause