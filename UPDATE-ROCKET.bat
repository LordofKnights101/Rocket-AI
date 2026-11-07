@echo off
title ROCKET Updater

cd /d "Z:\Coding\Ai Helper (ROCKET)"

rem Find cargo: try PATH first, then the default rustup location.
set "CARGO=cargo"
cargo --version >nul 2>&1
if errorlevel 1 set "CARGO=%USERPROFILE%\.cargo\bin\cargo.exe"
"%CARGO%" --version >nul 2>&1
if errorlevel 1 (
    echo Rust/Cargo was not found on this PC - install it from https://rustup.rs
    pause
    exit /b 1
)

echo Updating ROCKET to the latest build...
echo =======================================

rem [1/4] The running app locks its exe - close ALL instances first.
echo [1/4] Closing any running ROCKET...
tasklist /FI "IMAGENAME eq rocket-ai.exe" 2>nul | find /I "rocket-ai.exe" >nul
if not errorlevel 1 (
    taskkill /F /IM rocket-ai.exe >nul 2>&1
    rem Give Windows a moment to release the exe lock.
    timeout /t 4 /nobreak >nul
)

echo [2/4] Building release version...
"%CARGO%" build --release
if errorlevel 1 (
    rem The exe lock can lag behind a just-killed process - retry once.
    timeout /t 3 /nobreak >nul
    "%CARGO%" build --release
    if errorlevel 1 (
        echo.
        echo Build FAILED - restarting your previous app untouched.
        start "" "target\release\rocket-ai.exe"
        pause
        exit /b 1
    )
)

echo [3/4] Building debug version...
"%CARGO%" build

echo [4/4] Starting the updated ROCKET...
start "" "target\release\rocket-ai.exe"
echo.
echo ROCKET is updated and running with the latest build!
echo.
timeout /t 3 /nobreak >nul
exit
