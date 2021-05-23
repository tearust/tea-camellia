#!/usr/bin/env bash

set -e

echo "*** Start build tearust/tea-camellia:latest ***"
bash ./docker/build.sh

mkdir -p tmp
cp ./docker/target/release/tea-camellia tmp
cp Dockerfile tmp
cd tmp
docker build -t tearust/tea-camellia:latest .
cd ..
rm -rf tmp

if [ -n "$1" ]; then
    docker tag tearust/tea-camellia:latest tearust/tea-camellia:$1
    docker push tearust/tea-camellia:$1
fi