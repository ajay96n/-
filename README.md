# League Reveal Console

A standalone Rust console application that automatically opens op.gg multi-search links and auto-accepts ready checks in League of Legends.

## Features

- **Auto Open Multi**: Automatically opens op.gg multi-search links when champion select starts
- **Auto Accept**: Automatically accepts ready checks with a 1-second delay
- **Hardcoded Configuration**: 
  - Multi provider is fixed to op.gg (no option to change)
  - Auto open multi is always enabled
  - Auto accept is always enabled
  - Accept delay is set to 2000ms (1 second before timer expires)

## Requirements

- League of Legends client must be running
- Linux environment (tested on Ubuntu)
- Rust 1.82+ (if building from source)

## Building

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Install system dependencies
sudo apt-get update
sudo apt-get install -y libssl-dev pkg-config

# Build the application
cargo build --release
```

## Usage

1. Start League of Legends client
2. Run the application:
   ```bash
   ./target/release/league-reveal-console
   ```

The application will:
- Wait for League Client to start
- Connect to the League Client API
- Monitor game state changes
- Automatically open op.gg multi-search when champion select begins
- Automatically accept ready checks

## Configuration

The application has hardcoded settings as requested:
- `AUTO_OPEN_MULTI`: `true` (always enabled)
- `AUTO_ACCEPT`: `true` (always enabled)  
- `ACCEPT_DELAY`: `2000` (1 second before timer expires)
- `MULTI_PROVIDER`: `"opgg"` (fixed to op.gg, no option to change)

## How it Works

1. **Process Detection**: Scans for League Client processes and extracts connection information
2. **API Connection**: Connects to the League Client API using the extracted credentials
3. **State Monitoring**: Polls the gameflow state to detect when champion select starts
4. **Auto Actions**: 
   - Opens op.gg multi-search link with all participants when champion select begins
   - Accepts ready checks automatically with a 1-second delay

## File Structure

```
src/
├── main.rs           # Main application logic
├── lcu_client.rs     # League Client API client
├── lobby.rs          # Lobby/participant data structures
├── summoner.rs       # Summoner data structures  
├── region.rs         # Region information
├── utils.rs          # Utility functions for creating links
├── champ_select.rs   # Champion select data structures
└── analytics.rs      # Analytics (disabled in console version)
```

## Notes

- This is a console application with no GUI
- All configuration is hardcoded as requested
- The application will continuously run until manually stopped
- Requires League of Legends client to be running to function