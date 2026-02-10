# Cazzmachine

> la macchina che cazzeggia per te

Cazzmachine is a desktop application that automatically browses distracting content (memes, jokes, news, videos, gossip) for you, so you don't have to. It simulates doomscrolling behavior in the background while you work, then presents you with a curated selection at the end of your work session.

![Cazzmachine](public/doomscrolling.png)

## Features

- **Autonomous Content Consumption**: Automatically crawls entertaining content from various sources
- **Smart Buffering**: Maintains a buffer of ~20 unconsumed items, downloading more only when needed
- **Consumption Simulation**: Simulates realistic time costs for consuming different content types
- **Doomscroll Control**: Adjustable intensity knob (1-10) controls phase duration and crawl frequency
- **Daily Pruning**: Automatically cleans old data to minimize storage footprint
- **Terminal Aesthetic**: Clean, monospace-heavy UI throughout

## Architecture

The app consists of two main parts:

1. **Rust Backend** (`src-tauri/`)
   - CrawlScheduler: Downloads content when buffer is low (< 20 items)
   - NotificationEngine: Triggers doomscrolling phases at intervals
   - SQLite Database: Stores items with consumption state

2. **React Frontend** (`src/`)
   - IdleView: Main dashboard with knob and stats
   - DetailView: Browse consumed items
   - Summary: End-of-session report

## Prerequisites

- **Node.js** 18+ with npm
- **Rust** 1.70+ (install from [rustup.rs](https://rustup.rs))
- **Tauri CLI**: `cargo install tauri-cli` (optional, or use npm scripts)

## Development

Clone the repository:

```bash
git clone https://github.com/diego-conti/cazzmachine.git
cd cazzmachine
```

Install dependencies:

```bash
npm install
```

Run in development mode:

```bash
npm run tauri dev
```

This starts both the Vite dev server and the Tauri app with hot reload.

## Building from Source

### Desktop (Linux)

```bash
npm run tauri build
```

Output locations:
- **Debian package**: `src-tauri/target/release/bundle/deb/*.deb`
- **AppImage**: `src-tauri/target/release/bundle/appimage/*.AppImage`
- **Binary**: `src-tauri/target/release/cazzmachine`

Install the Debian package:

```bash
sudo dpkg -i src-tauri/target/release/bundle/deb/cazzmachine_*.deb
# Fix any dependency issues:
sudo apt-get install -f
```

Or run the AppImage directly:

```bash
chmod +x src-tauri/target/release/bundle/appimage/cazzmachine_*.AppImage
./src-tauri/target/release/bundle/appimage/cazzmachine_*.AppImage
```

### macOS

```bash
npm run tauri build
```

Output:
- **DMG**: `src-tauri/target/release/bundle/dmg/*.dmg`
- **App**: `src-tauri/target/release/bundle/macos/*.app`

### Windows

```bash
npm run tauri build
```

Output:
- **Installer**: `src-tauri/target/release/bundle/msi/*.msi`
- **Executable**: `src-tauri/target/release/cazzmachine.exe`

### Android

Prerequisites:
- **Android Studio** with SDK, NDK, and Command-line Tools installed
- Or install SDK manually: https://developer.android.com/studio#command-line-tools

Build from source:

```bash
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/26.1.10909125
npm run tauri android build
```

To install on your device:

```bash
# Enable "Install from unknown sources" in Android settings
adb install path/to/cazzmachine-signed.apk
```

## Project Structure

```
cazzmachine/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── hooks/              # Custom React hooks
│   ├── stores/             # Zustand state management
│   └── lib/                # Tauri API wrappers
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── crawler/        # Content providers (Reddit, etc.)
│   │   ├── db/             # Database models and queries
│   │   ├── notifications/  # Timer and event emission
│   │   └── commands.rs     # Tauri command handlers
│   ├── migrations/         # Database migrations
│   └── icons/              # App icons
├── public/                 # Static assets
└── package.json
```

## Configuration

The app stores data in:
- **Linux**: `~/.local/share/com.cazzmachine.app/`
- **macOS**: `~/Library/Application Support/com.cazzmachine.app/`
- **Windows**: `%APPDATA%/com.cazzmachine.app/`
- **Android**: Internal storage at `/data/data/com.cazzmachine.app/`

Database file: `cazzmachine.db`

## Consumption Costs

Different content types have different "time costs":

| Type | Cost (minutes) |
|------|----------------|
| Meme | 0.5 |
| Joke | 0.3 |
| News | 2.0 |
| Video | 3.0 |
| Gossip | 1.5 |

During a doomscrolling phase, items are consumed until the budget (phase duration) is exhausted. Unconsumed items remain in the buffer for future phases.

## Doomscroll Levels

The knob (1-10) controls:

| Level | Crawl Interval | Providers/Cycle | Phase Duration |
|-------|---------------|-----------------|----------------|
| 1 (low) | 15 min | 1 | 60s |
| 5 (mid) | ~11 min | 2 | ~167s |
| 10 (high) | 2 min | 3 | 300s |

## License

MIT

## Acknowledgments

Built with:
- [Tauri](https://tauri.app/) - Rust-based desktop framework
- [React](https://react.dev/) - UI library
- [Tailwind CSS](https://tailwindcss.com/) - Styling
- [Zustand](https://github.com/pmndrs/zustand) - State management
