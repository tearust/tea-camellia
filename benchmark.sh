#!/usr/bin/env bash

PRIME_COMMAND=$1
: ${PRIME_COMMAND:=""}

if [ $PRIME_COMMAND = "build" ]; then
    cargo build --release --features runtime-benchmarks
elif [ $PRIME_COMMAND = "test" ]; then
    cargo test --features runtime-benchmarks
elif [ $PRIME_COMMAND = "weight" ]; then
    PALLET_NAME=$2
    : ${PALLET_NAME:="pallet_tea"}
    OUTPUT_PATH=$3
    : ${OUTPUT_PATH:="pallet_tea.rs"}
    EXTRINSI_NAME=$4
    : ${EXTRINSI_NAME:="*"}
    BENCHMARK_STEP=$5
    : ${BENCHMARK_STEP:=50}
    BENCHMARK_REPEAT=$6
    : ${BENCHMARK_REPEAT:=20}

    # params:
    #--chain            # Configurable Chain Spec
    #--execution        # Always test with Wasm
    #--wasm-execution   # Always used `wasm-time`
    #--pallet           # Select the pallet
    #--extrinsic        # Select the extrinsic
    #--steps            # Number of samples across component ranges
    #--repeat           # Number of times we repeat a benchmark
    #--output           # Output benchmark results into a folder or file
    ./target/release/tea-camellia benchmark --chain dev --execution=wasm --wasm-execution=compiled --pallet "$PALLET_NAME" --extrinsic "$EXTRINSI_NAME" --steps $BENCHMARK_STEP --repeat $BENCHMARK_REPEAT --output "$OUTPUT_PATH"
else
    echo "unknown command, supported commands: build, test, weight"
fi