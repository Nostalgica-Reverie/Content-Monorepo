#!/bin/bash
echo Refreshing
(cd ./modpacks/simply-legacy/mr/yarn && pw batch refresh -y) &
(cd ./modpacks/simply-legacy/cf/yarn && pw batch refresh -y) &
(cd ./modpacks/re-console-plus/mr/yarn && pw batch refresh -y) &
(cd ./modpacks/re-console-plus/cf/yarn && pw batch refresh -y) &
(cd ./modpacks/2000s-edition/cf/yarn && pw batch refresh -y) &
(cd ./modpacks/2000s-edition/mr/yarn && pw batch refresh -y) &
(cd ./modpacks/rekindled-legacy/mr/forged/1.21.1-neoforge && pw batch refresh -y) &
(cd ./modpacks/rekindled-legacy/cf/forged/1.21.1-neoforge && pw batch refresh -y) &
wait
echo Done