#!/bin/bash

# use libafb development version if any
export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
export PATH="/usr/local/lib64:$PATH"
clear

if ! test -f $HOME/.cargo/build/debug/examples/libafb_sockcan.so; then
    cargo build --target-dir=$HOME/.cargo/build
    if test $? != 0; then
        echo "FATAL: fail to compile libafb sample"
        exit 1
    fi
fi

# rebuilt test binding
cargo build --target-dir=$HOME/.cargo/build --example tap_sockcan
if test $? != 0; then
    echo "FATAL: fail to compile test suite"
    exit 1
fi

# start binder with test config
afb-binder --config=examples/test/etc/binding-test-tap.json
