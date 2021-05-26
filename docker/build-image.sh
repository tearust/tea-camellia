#!/usr/bin/env bash
TEA_BUILD_MODE=$1
: ${TEA_BUILD_MODE:="normal"}

set -e

echo "*** Start build tearust/tea-camellia:latest ***"
bash ./docker/build.sh $TEA_BUILD_MODE

mkdir -p tmp
cp ./docker/target/release/tea-camellia tmp
cp Dockerfile tmp
cd tmp
docker build -t tearust/tea-camellia:latest .
cd ..
rm -rf tmp

if [ -n "$2" ]; then
    echo "*** Start tag and push tearust/tea-camellia:$2 ***"
    docker tag tearust/tea-camellia:latest tearust/tea-camellia:$2
    docker push tearust/tea-camellia:$2
fi
set +e