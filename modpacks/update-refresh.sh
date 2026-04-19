#!/bin/bash
set -eu

PACKS=("simply" "rc-plus" "2k" "rekindled")

echo "Updating..."

pids=()
for pack in "${PACKS[@]}"; do
    pack_dir="modpacks/$pack"
    if [ ! -d "$pack_dir" ]; then
        echo "warning: $pack_dir missing, skipping"
        continue
    fi

    for subdir in "$pack_dir"/*-mr "$pack_dir"/*-cf; do
        [ -d "$subdir" ] || continue
        (
            echo "[$subdir] updating"
            if (cd "$subdir" && packwiz update -a -y); then
                echo "[$subdir] ok"
            else
                echo "[$subdir] FAIL" >&2
                exit 1
            fi
        ) &
        pids+=($!)
    done
done

fail=0
for pid in "${pids[@]}"; do
    wait "$pid" || fail=$((fail + 1))
done

if [ "$fail" -gt 0 ]; then
    echo "$fail subdir(s) failed" >&2
    exit 1
fi

echo "Done"
