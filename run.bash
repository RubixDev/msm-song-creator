#!/bin/bash

for i in {1..21}; do
    if [ "$i" -eq 11 ] || [ "$i" -eq 20 ]; then continue; fi

    island="$(printf "%02d" "$i")"
    echo -e "\x1b[36mIsland \x1b[1m$island\x1b[22m...\x1b[0m"
    target/release/msm-song-creator "$island"
    echo -e "\x1b[32m..done\x1b[0m"
done
