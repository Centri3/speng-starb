#!/usr/bin/env bash

if [ $# -eq 0 ]
  then
    echo "SE directory not provided"
    exit 1
fi

mold -run cargo build -Zunstable-options --out-dir=target/build --target=x86_64-pc-windows-gnu
cp target/build/speng_starb_proxy.dll "$1"/version.dll
cp target/build/speng_starb.dll "$1"/speng_starb.dll
