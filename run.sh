#!/bin/bash

cat state > state.tmp

if [ "$(cat state.tmp)" = "" ]; then
    cargo run | tee state
else
    cat state.tmp | ./target/release/cliciv "$@" > state
fi
