#!/bin/bash
# Vanilla Modpacks
vpacks=(
    re-console
    re-console-lite
)
# Modded Packs
mpacks=(
    arics-console
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
                    echo
                    echo Updating $i $q $w $e
                    (cd $DIR && packwiz refresh -y && packwiz update -a -y)
                fi
            done
        if [ -d "./versions/vanilla/src/$i/$q/$w" ]; then
            echo
            echo Waiting 20s for rate-limiting
            echo
            sleep 20
        fi
        done
    done
done
echo ---------------------
echo Vanilla Packs Updated
echo ---------------------
for i in "${mpacks[@]}"; do
    for q in "${pfvs[@]}"; do
        for e in "${mcvs[@]}"; do
            DIR="./versions/modded/src/$i/$q/$e"
            if [ -d "$DIR" ]; then
                echo
                echo Updating $i $q $w $e
                (cd $DIR && packwiz refresh -y && packwiz update -a -y)
            fi
        done
    if [ -d "./versions/modded/src/$i/$q" ]; then
        echo
        echo Waiting 20s for rate-limiting
        echo
        sleep 20
    fi
    done
done
echo --------------------
echo Modded Packs Updated
echo --------------------