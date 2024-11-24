#!/bin/bash
sudo apt update && sudo apt install -y musl-tools

export PNPM_HOME=$HOME/.pnpm-global
export PNPM_STORE=$HOME/.pnpm-store
export PATH="$HOME/.pnpm-global/bin:$PATH"
export CARGO_TARGET_DIR=$HOME/.cargo/target

# Create necessary directories if they do not exist
mkdir -p $CARGO_TARGET_DIR
mkdir -p $PNPM_STORE
mkdir -p $PNPM_HOME/bin

# Set ownership to the current user
sudo chown -R $(whoami):$(whoami) $CARGO_TARGET_DIR $PNPM_STORE $PNPM_HOME

# Set permissions to allow read/write/execute for the user
chmod -R u+rwx $CARGO_TARGET_DIR $PNPM_STORE $PNPM_HOME

cargo install cargo-watch

mkdir -p $PNPM_HOME/bin

pnpm config set prefix $PNPM_HOME
pnpm config set shared-store true
pnpm config set global-bin-dir $PNPM_HOME/bin
pnpm config set store-dir $PNPM_STORE

if [ -z "$(ls -A $PNPM_STORE)" ]; then
    pnpm setup
fi

pnpm install --global --save-dev webpack webpack-cli

echo 'export PNPM_HOME=$HOME/.pnpm-global' >> ~/.bashrc
echo 'export PNPM_STORE=$HOME/.pnpm-store' >> ~/.bashrc
echo 'export PATH="$HOME/.pnpm-global/bin:$PATH"' >> ~/.bashrc
echo 'export CARGO_TARGET_DIR=$HOME/.cargo/target' >> ~/.bashrc