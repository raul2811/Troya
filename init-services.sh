#!/bin/bash
# init-services.sh
docker-compose down
docker volume rm -f sistema_bancarario_hnl_tigerbeetle_data 2>/dev/null || true
docker-compose run --rm tigerbeetle format --cluster=0 --replica=0 --replica-count=1 /data/0_0.tigerbeetle
docker-compose up -d