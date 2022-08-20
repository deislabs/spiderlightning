#!/bin/bash

## update and install 1st level of packages
apt-get update
apt-get install -y \
    curl \
    git \
    gnupg2 \
    jq \
    sudo \
    zsh \
    build-essential \
    cmake \
    libssl-dev \
    openssl \
    unzip

## update and install 2nd level of packages
apt-get install -y pkg-config

## install rustup and common components
curl https://sh.rustup.rs -sSf | sh -s -- -y

rustup install nightly
rustup component add rustfmt
rustup component add rustfmt --toolchain nightly
rustup component add clippy
rustup component add clippy --toolchain nightly

cargo install cargo-expand
cargo install cargo-edit
cargo install --git https://github.com/bytecodealliance/wit-bindgen wit-bindgen-cli --rev a79a4be33d76ddf62839ba71602c26a96610ef7c
rustup target add wasm32-wasi

## setup git
git config --global core.editor "code --wait"
