#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

echo "*** Start build tea camellia ***"

cd $(dirname ${BASH_SOURCE[0]})/..

docker compose down --remove-orphans
docker compose run --rm build $@
