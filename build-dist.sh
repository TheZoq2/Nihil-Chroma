#!/bin/sh

set -e

cargo rustc --release -- -C link-args='-Wl,-rpath,$ORIGIN/'
mkdir dist
cp target/release/nihil_chroma dist
# Change to wherever the libraries you want to bundle are located
cp /usr/lib/libSDL2-2.0.so.0 dist
cp /usr/lib/libSDL2_ttf-2.0.so.0 dist
cp /usr/lib/libSDL2_image-2.0.so.0 dist
cp /usr/lib/libSDL2_mixer-2.0.so.0 dist
cp -r data dist
