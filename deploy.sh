#!/bin/bash

if [ ! -d "./chess" ]; then
	git clone https://appellation:$GITHUB_TOKEN@github.com/appellation/chess.git
	cd chess
else
	cd chess
	git fetch origin
	git reset --hard origin/master
fi

mv -f ../.env .

docker-compose pull
docker-compose build
docker-compose up -d
