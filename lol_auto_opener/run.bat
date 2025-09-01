@echo off
echo 🎮 Starting League of Legends Auto Opener...
echo Make sure League of Legends client is running!
echo.

REM Build if needed
if not exist "target\release\lol_auto_opener.exe" (
    echo 🔨 Building application...
    cargo build --release
)

REM Run the application
echo 🚀 Starting application...
target\release\lol_auto_opener.exe
pause