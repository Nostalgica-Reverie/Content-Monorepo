#!/bin/bash
echo Updating
(cd ./modpacks/simply && pw batch update -a -y && pw batch refresh -y) &
(cd ./modpacks/rc-plus && pw batch update -a -y && pw batch refresh -y) &
(cd ./modpacks/2k && pw batch update -a -y && pw batch refresh -y) &
(cd ./modpacks/rekindled && pw batch update -a -y && pw batch refresh -y) &
wait
echo Done