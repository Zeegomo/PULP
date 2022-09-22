#!/usr/bin/env bash
set -e

scriptDir=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )

source /gap_sdk/configs/ai_deck.sh
make clean all

cd ./BUILD/GAP8_V2/GCC_RISCV_FREERTOS
ar crus libwrapper.a wrapper.o
