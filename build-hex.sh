#!/usr/bin/env bash
set -euo pipefail

# Always build from the repo
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

# Replace w new hex after compile and conversion
hex_tmp=$(mktemp ./vcu.hex.XXXXXX)
trap 'rm -f -- "$hex_tmp"' EXIT
cargo objcopy --locked --release --bin sdm-vcu-firm -- -O ihex "$hex_tmp"
chmod 644 "$hex_tmp"
mv -f -- "$hex_tmp" vcu-firm.hex
printf 'Built %s/vcu-firm.hex\n' "$PWD"
