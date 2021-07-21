#!/bin/bash

function log() {
  docker-compose logs -f
}

function start() {
  docker-compose up -d
  log
}

function stop() {
  docker-compose down
}


if [ $1 == "refresh" ]; then
  set -x

  stop
  rm -rf .layer1/share/tea-camellia/chains/local_testnet/db
  start

  set +x
elif [ $1 == "start" ]; then
  start
elif [ $1 == "stop" ]; then
  stop
elif [ $1 == "log" ]; then
  log
else
  echo "unknown command. Supported subcommand: start, stop, refresh, log"
fi