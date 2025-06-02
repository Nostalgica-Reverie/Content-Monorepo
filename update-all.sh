#!/bin/bash
# Vanilla Modpacks
vpacks=(
    re-console
    re-console-lite
)
# Modded Packs
mpacks=(
    arics-console
    williams-console-revisited
)
# Platform Version
pfvs=(
    curseforge
    modrinth
)
# Mod Loader Version
mlvs=(
    fabric
    neoforged
)
# Minecraft Version
mcvs=(
    1.20.1
    1.21.1
    1.21.4
    1.21.5
)

for i in "${vpacks[@]}"; do
    for q in "${pfvs[@]}"; do
        for w in "${mlvs[@]}"; do
            for e in "${mcvs[@]}"; do
                DIR="./versions/vanilla/src/$i/$q/$w/$e"
                if [ -d "$DIR" ]; then
                    echo Updating $i $q $w $e
                    (cd $DIR && packwiz update -a -y)
                fi
            done
        done
    done
    echo Waiting 10s for rate-limiting
    sleep 10
done
echo
echo Vanilla Packs Updated
echo
for i in "${mpacks[@]}"; do
    for q in "${pfvs[@]}"; do
        for e in "${mcvs[@]}"; do
            DIR="./versions/modded/src/$i/$q/$e"
            if [ -d "$DIR" ]; then
                echo Updating $i $q $w $e
                (cd $DIR && packwiz update -a -y)
            fi
        done
    done
    echo Waiting 10s for rate-limiting
    sleep 10
done
echo
echo Modded Packs Updated