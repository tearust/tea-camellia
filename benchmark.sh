#!/usr/bin/env bash

PRIME_COMMAND=$1
: ${PRIME_COMMAND:=""}

function generate_weight() {
    # params:
    #--chain            # Configurable Chain Spec
    #--execution        # Always test with Wasm
    #--wasm-execution   # Always used `wasm-time`
    #--pallet           # Select the pallet
    #--extrinsic        # Select the extrinsic
    #--steps            # Number of samples across component ranges
    #--repeat           # Number of times we repeat a benchmark
    #--header           # Specify header file to insert into head of output file
    #--output          # Output benchmark results into a folder or file
    ./target/debug/tea-camellia benchmark \
        --chain=dev \
        --execution=wasm \
        --wasm-execution=compiled \
        --pallet="$PALLET_NAME" \
        --extrinsic="$EXTRINSI_NAME" \
        --steps=$BENCHMARK_STEP \
        --repeat=$BENCHMARK_REPEAT \
        --heap-pages=4096 \
        --header=./file_header.txt \
        --output="runtime/src/weights/${PALLET_NAME}.rs"
}

function generate_template() {
    # params:
    #--chain            # Configurable Chain Spec
    #--execution        # Always test with Wasm
    #--wasm-execution   # Always used `wasm-time`
    #--pallet           # Select the pallet
    #--extrinsic        # Select the extrinsic
    #--steps            # Number of samples across component ranges
    #--repeat           # Number of times we repeat a benchmark
    #--header           # Specify header file to insert into head of output file
    #--output           # Output benchmark results into a folder or file
    #--template         # Template file to generate `WeightInfo` trait and implement
    ./target/release/tea-camellia benchmark \
        --chain=dev \
        --execution=wasm \
        --wasm-execution=compiled \
        --pallet="$PALLET_NAME" \
        --extrinsic="$EXTRINSI_NAME" \
        --steps=$BENCHMARK_STEP \
        --repeat=$BENCHMARK_REPEAT \
        --heap-pages=4096 \
        --header="./file_header.txt" \
        --output="pallets/${PACKAGE_NAME}/src/weights.rs" \
        --template="./.maintain/frame-weight-template.hbs"
}

if [ $PRIME_COMMAND = "build" ]; then
    cargo build --release --features runtime-benchmarks
elif [ $PRIME_COMMAND = "test" ]; then
    cargo test --features runtime-benchmarks
elif [ $PRIME_COMMAND = "weight" ]; then
    PALLET_NAME=$2
    : ${PALLET_NAME:="pallet_tea"}
    EXTRINSI_NAME=$3
    : ${EXTRINSI_NAME:="*"}
    BENCHMARK_STEP=$4
    : ${BENCHMARK_STEP:=50}
    BENCHMARK_REPEAT=$5
    : ${BENCHMARK_REPEAT:=20}

    generate_weight
elif [ $PRIME_COMMAND = "template" ]; then
    PALLET_NAME=$2
    : ${PALLET_NAME:="pallet_tea"}
    PACKAGE_NAME=$3
    : ${PACKAGE_NAME:="tea"}
    EXTRINSI_NAME=$4
    : ${EXTRINSI_NAME:="*"}
    BENCHMARK_STEP=$5
    : ${BENCHMARK_STEP:=50}
    BENCHMARK_REPEAT=$6
    : ${BENCHMARK_REPEAT:=20}

    generate_template
elif [ $PRIME_COMMAND = "batch_weight" ]; then
    EXTRINSI_NAME="*"
    BENCHMARK_STEP=$2
    : ${BENCHMARK_STEP:=50}
    BENCHMARK_REPEAT=$3
    : ${BENCHMARK_REPEAT:=20}

    PALLET_NAME=frame_system && generate_weight
    PALLET_NAME=pallet_grandpa && generate_weight
    PALLET_NAME=pallet_timestamp && generate_weight
    PALLET_NAME=pallet_balances && generate_weight
    PALLET_NAME=pallet_babe && generate_weight
    PALLET_NAME=pallet_session && generate_weight
    PALLET_NAME=pallet_staking && generate_weight
    PALLET_NAME=pallet_offences && generate_weight
    PALLET_NAME=pallet_im_online && generate_weight
    PALLET_NAME=pallet_elections_phragmen && generate_weight
    PALLET_NAME=pallet_election_provider_multi_phase && generate_weight
    PALLET_NAME=pallet_collective && generate_weight
    PALLET_NAME=pallet_membership && generate_weight
    PALLET_NAME=pallet_scheduler && generate_weight
    PALLET_NAME=pallet_democracy && generate_weight
    PALLET_NAME=pallet_utility && generate_weight
    PALLET_NAME=pallet_multisig && generate_weight
    PALLET_NAME=pallet_identity && generate_weight
    PALLET_NAME=pallet_treasury && generate_weight

    PALLET_NAME=pallet_tea && generate_weight
    PALLET_NAME=pallet_cml && generate_weight
    PALLET_NAME=pallet_auction && generate_weight
else
    echo "unknown command, supported commands: build, test, weight, template"
fi
