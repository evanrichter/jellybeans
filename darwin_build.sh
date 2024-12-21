#!/bin/bash

set -euf

SDKROOT=$(xcrun -sdk macosx --show-sdk-path) cargo build --release --target=aarch64-apple-darwin
rm -rf build/macos/src/jellybeans.app/Contents/MacOS/assets
rm -rf build/macos/src/Applications
rm -rf jellybeans.dmg

pushd build/macos
./create_icns.sh
popd

mkdir -p build/macos/src/jellybeans.app/Contents/MacOS/assets
cp -r assets/ build/macos/src/jellybeans.app/Contents/MacOS/assets
cp target/aarch64-apple-darwin/release/jellybeans build/macos/src/jellybeans.app/Contents/MacOS/
ln -s /Applications build/macos/src/
hdiutil create -fs HFS+ -volname jellybeans -srcfolder build/macos/src jellybeans.dmg
