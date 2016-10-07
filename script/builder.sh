#!/bin/bash

set -e

BUILD_CMD="cargo build --release --verbose"

if [ "$BUILD_DOCKER" == "1" ]; then
	BUILD_CMD="docker run -ti --rm -v $PWD:/source cattha/rust:nightly script/builder.sh"
fi

$BUILD_CMD

bins=$(find target/release -maxdepth 1 -type f -executable)

mkdir -p output/bin output/lib64

for bin in $bins; do
	script/get_libs.sh $bin output/lib64
	cp $bin output/bin/
done
