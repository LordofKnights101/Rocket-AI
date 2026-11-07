#!/bin/bash

cd "$(dirname "$0")"
echo "Starting ROCKET AI Companion..."
echo "=============================="
echo ""
echo "ROCKET is your personal gaming AI companion built with Rust!"
echo "Features:"
echo "• Game-specific cheat sheets and knowledge"
echo "• Animated rocket mascot with flame effects"
echo "• Online AI API integration (Groq, Gemini, OpenAI)"
echo "• Local AI mode for offline use"
echo "• Multi-game support (Minecraft, Roblox, PUBG, and more!)"
echo ""
echo "=============================="
echo ""

./target/release/rocket-ai &

echo "ROCKET AI Companion has been launched!"
echo ""
echo "You can now use the application. It will appear in your taskbar/dock."
echo ""
echo "To close the application, simply close the window or run:"
echo "pkill -f rocket-ai"
echo ""
read -p "Press Enter to continue..."