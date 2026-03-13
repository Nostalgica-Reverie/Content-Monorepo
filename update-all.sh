#!/bin/bash
echo Updating
(cd ./modpacks/simply-legacy/mr/yarn/1.21.10-fabric && packwiz update -a -y) &
(cd ./modpacks/simply-legacy/cf/yarn/1.21.10-fabric && packwiz update -a -y) &
(cd ./modpacks/re-console+/mr/yarn/1.21.10-fabric && packwiz update -a -y) &
(cd ./modpacks/re-console+/cf/yarn/1.21.10-fabric && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/b0033 && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/b0035 && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/b0054 && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/gdc2012 && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu0 && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu1 && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu2 && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu3 && packwiz update -a -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge/tu5 && packwiz update -a -y) &
wait
echo Done