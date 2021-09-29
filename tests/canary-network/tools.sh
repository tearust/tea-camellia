#!/bin/bash

if [ $1 = "tunnel" ]; then
    SSH_HOST=$2
    : ${SSH_HOST:="pc"}

    ssh \
	    -L 9944:127.0.0.1:9944 \
	    -L 9933:127.0.0.1:9933 \
                               \
	    -L 9943:127.0.0.1:9943 \
	    -L 9942:127.0.0.1:9942 \
	    -L 9941:127.0.0.1:9941 \
	    -L 9940:127.0.0.1:9940 \
	    -L 9939:127.0.0.1:9939 \
	    -L 9938:127.0.0.1:9938 \
	    -N -T $SSH_HOST
elif [ $1 = "clean" ]; then
	rm -rf .layer1/alice/share/tea-camellia/chains/tea-layer1/db
	rm -rf .layer1/bob/share/tea-camellia/chains/tea-layer1/db
	rm -rf .layer1/charlie/share/tea-camellia/chains/tea-layer1/db
	rm -rf .layer1/dave/share/tea-camellia/chains/tea-layer1/db
	rm -rf .layer1/ferdie/share/tea-camellia/chains/tea-layer1/db
	rm -rf .layer1/eve/share/tea-camellia/chains/tea-layer1/db
	rm -rf .layer1/george/share/tea-camellia/chains/tea-layer1/db
else
    echo "unknown command. Supported subcommand: tunnel"
fi
