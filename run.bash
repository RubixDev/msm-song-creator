#!/bin/bash

for i in {1..21}; do
    if [ "$i" -eq 11 ] || [ "$i" -eq 20 ]; then continue; fi

    echo -e "\x1b[36mIsland \x1b[1m$i\x1b[22m...\x1b[0m"
    target/release/msm "$i" -o songs
    echo -e "\x1b[32m..done\x1b[0m"
done
