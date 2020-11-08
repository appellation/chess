#!/bin/bash

git fetch origin
git reset --hard origin/master

mv -f ../.env .

./run.sh prod pull
./run.sh prod build
./run.sh prod up -d
