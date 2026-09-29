#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    printf 'Usage: bash flash.sh <firmware-elf>\n' >&2
    exit 1
fi

for tool in rust-objcopy teensy_loader_cli; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf 'Missing PATH dependency: %s\n' "$tool" >&2
        exit 1
    fi
done

hex_tmp=$(mktemp "${TMPDIR:-/tmp}/vcu-flash.XXXXXX")
trap 'rm -f -- "$hex_tmp"' EXIT
rust-objcopy -O ihex "$1" "$hex_tmp"
printf 'Waiting for Teensy 4.1...\n'
teensy_loader_cli --mcu=TEENSY41 -w -v "$hex_tmp"

