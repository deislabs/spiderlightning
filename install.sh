#!/bin/bash

INSTALL_DIR="/usr/local"
OWNER_AND_REPO="deislabs/spiderlightning"
TAR="slight-unix.tar.gz"
BINARY_NAME="slight"

LATEST_RELEASE="$(curl -s https://api.github.com/repos/$OWNER_AND_REPO/releases | grep tag_name | awk 'NR == 1' | cut -d : -f 2 | cut -d \" -f 2)"
echo ">>> LATEST RELEASE: $LATEST_RELEASE..."

URL="https://github.com/$OWNER_AND_REPO/releases/download/$LATEST_RELEASE/$TAR"
echo ">>> DONLOADING FROM: $URL..."

curl -L -s $URL --output $TAR
echo ">>> DOWNLOADED BINARY TAR."

tar -xf $TAR
echo ">>> EXTRACTED BINARY TAR."

sudo install ./release/$BINARY_NAME $INSTALL_DIR/bin
echo ">>> INSTALLED BINARY."

rm $TAR
sudo rm -rf ./release
echo ">>> CLEANED UP."