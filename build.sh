#!/bin/bash

TARGET="aarch64-unknown-none"

echo "Building..."
cargo build --release --target ${TARGET}

echo "Extracting kernel image..."
aarch64-none-elf-objcopy -O binary target/${TARGET}/release/kernel kernel.img

if [[ "${1:-}" == "qemu" ]]; then
    qemu-system-aarch64 \
        -M raspi3b \
        -m 1G \
        -drive file=./disk.img,if=sd,format=raw \
        -kernel kernel.img \
        -serial stdio
fi
