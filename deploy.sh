#!/bin/bash

git fetch origin
git reset --hard origin/master

mv -f ../.env .

docker-compose pull
docker-compose build
docker-compose up -d
