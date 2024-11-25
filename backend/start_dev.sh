#!/bin/bash
ROOT="$(dirname "$(realpath "$0")")/.."

export PORT=8080 
export ADDR=0.0.0.0 
export DATA_FOLDER=${ROOT}/data 
export PASSWORD=dev
export STATIC_DIR=../static

export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

cargo run