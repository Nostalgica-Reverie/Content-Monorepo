#!/bin/bash
echo Refreshing
(cd ./modpacks/simply-legacy/mr/yarn/1.21.10-fabric && packwiz refresh -y) &
(cd ./modpacks/simply-legacy/cf/yarn/1.21.10-fabric && packwiz refresh -y) &
(cd ./modpacks/re-console+/mr/yarn/1.21.10-fabric && packwiz refresh -y) &
(cd ./modpacks/re-console+/cf/yarn/1.21.10-fabric && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/b0033 && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/b0035 && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/b0054 && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/gdc2012 && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu0 && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu1 && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu2 && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu3 && packwiz refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu5 && packwiz refresh -y) &
wait
echo Done