#!/bin/bash
echo Updating
(cd ./modpacks/simply-legacy/mr/yarn/1.21.10-fabric && packwiz update -a -y) &
(cd ./modpacks/simply-legacy/cf/yarn/1.21.10-fabric && packwiz update -a -y) &
(cd ./modpacks/re-console-plus/mr/yarn/1.21.10-fabric && packwiz update -a -y) &
(cd ./modpacks/re-console-plus/cf/yarn/1.21.10-fabric && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge && pw batch update -a -y) &
(cd ./modpacks/rekindled-legacy/cf/forged/1.21.1-neoforge && pw batch update -a -y) &
wait
echo Done