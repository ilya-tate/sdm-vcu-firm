#!/bin/sh
set -e

if [ "$(basename "$1")" = sdm-vcu-firm ]; then
    echo "Board not connected, try cargo test -- --no-board" >&2
    exit 1
fi

if ! docker info >/dev/null 2>&1; then
    echo "Docker not found" >&2
    exit 1
fi

echo "Building test image..." >&2
docker build --quiet --tag sdm-vcu-no-board docker >/dev/null

docker run --rm --interactive --network none sdm-vcu-no-board "$@" <"$1"
