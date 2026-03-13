#!/bin/bash
echo Updating
(cd ./modpacks/simply-legacy/mr/yarn && pw batch update -a -y) &
(cd ./modpacks/simply-legacy/cf/yarn && pw batch update -a -y) &
(cd ./modpacks/re-console-plus/mr/yarn && pw batch update -a -y) &
(cd ./modpacks/re-console-plus/cf/yarn && pw batch update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge && pw batch update -a -y) &
(cd ./modpacks/rekindled-legacy/cf/forged/1.21.1-neoforge && pw batch update -a -y) &
wait
echo Done