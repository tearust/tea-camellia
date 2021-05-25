#!/bin/bash

if [ $1 = "tunnel" ]; then
    $HOSTNAME=$2
    : ${HOSTNAME:="pc"}

    ssh \
	    -L 9944:127.0.0.1:9944 \
	    -L 9933:127.0.0.1:9933 \
                               \
	    -L 9943:127.0.0.1:9943 \
	    -L 9942:127.0.0.1:9942 \
	    -L 9941:127.0.0.1:9941 \
	    -L 9940:127.0.0.1:9940 \
	    -L 9939:127.0.0.1:9939 \
	    -N -T $HOSTNAME
else
    echo "unknown command. Supported subcommand: tunnel"
fi
