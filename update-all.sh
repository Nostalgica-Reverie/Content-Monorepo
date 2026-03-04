#!/bin/bash
echo Updating
(cd ./modpacks/simply-legacy/mr/yarn/1.21.10-fabric && packwiz refresh -y && packwiz update -a -y)