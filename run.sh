#!/bin/bash

echo "🎮 Starting League of Legends Auto Opener..."
echo "Make sure League of Legends client is running!"
echo ""

# Build if needed
if [ ! -f "target/release/lol_auto_opener" ]; then
    echo "🔨 Building application..."
    cargo build --release
fi

# Run the application
echo "🚀 Starting application..."
./target/release/lol_auto_opener