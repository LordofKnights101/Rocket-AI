# ROCKET AI Companion - Quick Start Guide

## 🚀 Easy Launch Options

### Option 1: Desktop Shortcut (Recommended)
1. Right-click on `create_shortcut.ps1` and select "Run with PowerShell"
2. A shortcut will be created on your desktop
3. Double-click the desktop shortcut to launch ROCKET

### Option 2: Batch File (Windows)
1. Double-click `launch_rocket.bat` to start the application
2. Or run it from Command Prompt: `launch_rocket.bat`

### Option 3: Shell Script (Linux/macOS)
1. Run `./launch_rocket.sh` in the terminal
2. Or make it executable first: `chmod +x launch_rocket.sh` then `./launch_rocket.sh`

### Option 4: Direct Execution
1. Navigate to the application directory
2. Run `target\release\rocket-ai.exe` (Windows) or `./target/release/rocket-ai` (Linux/macOS)

## 📁 Application Files

- **Main executable**: `target\release\rocket-ai.exe`
- **Configuration**: Saved in `rocket-llm.json` (AI settings) and `rocket_chat_history.json` (chat history)
- **Launch scripts**: `launch_rocket.bat`, `launch_rocket.sh`, `create_shortcut.ps1`

## 🎮 Features

### Game Support
- **Minecraft**: Diamond farming, building tips, redstone guides
- **Roblox**: Game development, Robux earning, scripting basics
- **PUBG**: Vehicle strategy, looting tips, circle management, combat tactics
- **Valorant**: Weapon guides, agent abilities, aim tips, spike strategy
- **Skyrim**: Combat classes, skill progression, quest strategies
- **League of Legends**: Roles, champion types, win conditions
- **Fortnite**: Building tips, weapon guides, general strategy
- **And more**: Generic gaming tips for all other games

### AI Features
- **Local AI**: Built-in knowledge base, works offline
- **Online AI**: API integration with Groq, Gemini, OpenAI, OpenRouter
- **Game-specific responses**: Context-aware answers for each supported game
- **Cheat sheets**: Comprehensive guides accessible anytime

### User Interface
- **Animated rocket mascot**: Bobbing, flame effects, launch animations
- **Clean chat interface**: Easy-to-use messaging system
- **Game switching**: Quick selection between different games
- **Settings panel**: Configure AI providers and API keys

## 🔧 Setup Requirements

- **Windows**: Windows 10/11 with .NET Runtime (automatically handled by Rust)
- **Linux**: Standard X11 desktop environment
- **macOS**: macOS 10.14 or later

## 🚀 Getting Started

1. **Launch the application** using one of the methods above
2. **Select your game** from the dropdown menu
3. **Start chatting** with ROCKET for gaming assistance
4. **Configure AI** (optional): Click the settings button to enable online AI mode

## 📝 Usage Tips

- Type "help" for general gaming assistance
- Type "cheat sheet" to see game-specific guides
- Use "diamond", "building", "survival" etc. for game-specific tips
- Toggle between local and online AI modes in settings

## 🔧 Troubleshooting

- **Application won't start**: Ensure you're in the correct directory and have built the release version
- **AI not responding**: Check your internet connection for online mode, or switch to local mode
- **Settings not saving**: Ensure the application has write permissions to the directory

## 🎯 Next Steps

The application is ready to use! You can now:
- Explore game-specific cheat sheets
- Get AI-powered gaming assistance
- Customize AI settings
- Enjoy the animated rocket mascot

Happy gaming! 🎮🚀