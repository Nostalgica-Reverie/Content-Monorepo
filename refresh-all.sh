#!/bin/bash
echo Refreshing
(cd ./modpacks/simply-legacy/mr/yarn/1.21.10-fabric && packwiz refresh -y) &
(cd ./modpacks/simply-legacy/cf/yarn/1.21.10-fabric && packwiz refresh -y) &
(cd ./modpacks/re-console-plus/mr/yarn/1.21.10-fabric && packwiz refresh -y) &
(cd ./modpacks/re-console-plus/cf/yarn/1.21.10-fabric && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge && pw batch refresh -y) &
(cd ./modpacks/rekindled-legacy/cf/forged/1.21.1-neoforge && pw batch refresh -y) &
wait
echo Done