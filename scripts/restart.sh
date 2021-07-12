#!/bin/bash
CONTAINER_NAME="omSupply-backend-postgres"
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

docker stop ${CONTAINER_NAME}
docker rm ${CONTAINER_NAME}
${SCRIPT_DIR}/init_db.sh