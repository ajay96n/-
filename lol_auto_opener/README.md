# League of Legends Auto Opener Console Application

A standalone Rust console application that automatically opens OP.GG Multi when you enter champion select and auto-accepts ready checks.

## Features

- **Auto Open Multi**: Automatically opens OP.GG Multi with all team members when entering champion select
- **Auto Accept**: Automatically accepts ready checks
- **Console Interface**: Clean console output with status updates
- **Fixed Configuration**: 
  - Multi provider is hardcoded to OP.GG (cannot be changed)
  - Auto open is always enabled
  - Auto accept is always enabled

## Requirements

- League of Legends client must be running
- Rust toolchain (for building from source)

## Installation

1. Make sure you have Rust installed:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone or download this project

3. Build the application:
   ```bash
   cargo build --release
   ```

4. The executable will be available at `target/release/lol_auto_opener` (or `target/release/lol_auto_opener.exe` on Windows)

## Usage

1. Start League of Legends client
2. Run the application:
   ```bash
   cargo run
   ```
   Or run the compiled executable:
   ```bash
   ./target/release/lol_auto_opener
   ```

3. The application will:
   - Wait for League client to be detected
   - Monitor for ready checks and champion select
   - Automatically accept ready checks
   - Automatically open OP.GG Multi when entering champion select

## Configuration

The application has a hardcoded configuration:
- **Auto Open**: Always enabled
- **Auto Accept**: Always enabled  
- **Accept Delay**: 2000ms (2 seconds)
- **Multi Provider**: OP.GG (fixed, cannot be changed)

## Console Output

The application provides clear console feedback:
- ✅ Connection status
- 🎯 Champion select detection
- 🎮 Ready check handling
- 🌍 Region information
- 📋 Team member listing
- 🔗 OP.GG link opening

## How It Works

1. **League Client Detection**: Uses the `shaco` library to detect and connect to the League of Legends client
2. **WebSocket Connection**: Subscribes to League client events for real-time updates
3. **Game State Monitoring**: Tracks game flow phases (lobby, matchmaking, champion select, etc.)
4. **Automatic Actions**: 
   - When ready check is detected, automatically accepts after the configured delay
   - When champion select is detected, fetches team information and opens OP.GG Multi

## Dependencies

- `tokio`: Async runtime
- `serde`: Serialization/deserialization
- `serde_json`: JSON handling
- `futures-util`: Stream utilities
- `urlencoding`: URL encoding for OP.GG links
- `open`: Opening URLs in browser
- `shaco`: League of Legends client API integration

## Troubleshooting

- **"Waiting for League Client to open"**: Make sure League of Legends is running
- **Connection issues**: Try restarting both the application and League client
- **Browser not opening**: Check that you have a default browser set
- **Permission issues**: Make sure the application has permission to open URLs

## License

This project is provided as-is for educational and personal use.