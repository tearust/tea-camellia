#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
TEA_BUILD_MODE=$1
: ${TEA_BUILD_MODE:="normal"}
shift

set -e

cd $(dirname ${BASH_SOURCE[0]})/..

docker compose down --remove-orphans
if [ $TEA_BUILD_MODE == "normal" ]; then
    echo "*** Start build tea camellia: normal mode ***"
    docker compose run --rm build $@
elif [ $TEA_BUILD_MODE == "fast" ]; then
    echo "*** Start build tea camellia: fast mode ***"
    docker compose run --rm fast $@
else
    echo "unknown build mode, supported modes: normal, fast"
fi