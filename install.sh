#!/bin/bash

INSTALL_DIR="/usr/local"
OWNER_AND_REPO="${OWNER_AND_REPO:-deislabs/spiderlightning}"
BINARY_NAME="slight"

LATEST_RELEASE="$(curl -s https://api.github.com/repos/$OWNER_AND_REPO/releases | grep tag_name | awk 'NR == 1' | cut -d : -f 2 | cut -d \" -f 2)"
echo ">>> LATEST RELEASE: $LATEST_RELEASE..."

OS="$(uname)"
ARCH="$(uname -m)"
if [[ "${OS}" == "Linux" ]]
then
    TAR="slight-ubuntu.tar.gz"
elif [[ "${OS}" == "Darwin" ]]
then
  if [[ "${ARCH}" == "arm64" ]]
  then
    TAR="slight-macos-aarch64.tar.gz"
  else
    TAR="slight-macos-amd64.tar.gz"
  fi
else
  echo ">>> THIS INSTALLATION METHOD ONLY WORKS FOR MACOS AND LINUX."
  exit 1
fi

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