#!/bin/bash
modpacks=(
    re-console
    re-console-lite
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

for i in "${modpacks[@]}"; do
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
done