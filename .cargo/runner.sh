#!/bin/sh

case " $* " in
    *" --no-board "*) exec docker/build_docker.sh "$@" ;;
    *) exec probe-rs run --chip STM32G474RE --rtt-channel-mode no-block-skip "$@" ;;
esac
