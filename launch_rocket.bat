@echo off
cd /d "Z:\Coding\Ai Helper (ROCKET)"
echo Starting ROCKET AI Companion...
echo ==============================
echo.
echo ROCKET is your personal gaming AI companion built with Rust!
echo Features:
echo • Game-specific cheat sheets and knowledge
echo • Animated rocket mascot with flame effects
echo • Online AI API integration (Groq, Gemini, OpenAI)
echo • Local AI mode for offline use
echo • Multi-game support (Minecraft, Roblox, PUBG, and more!)
echo.
echo ==============================
echo.

start "ROCKET AI Companion" target\release\rocket-ai.exe

echo ROCKET AI Companion has been launched!
echo.
echo You can now use the application. It will appear in your taskbar.
echo.
echo To close the application, simply close the window or press Ctrl+C in this terminal.
echo.
pause