#!/bin/sh

SERVICE_DIR_NAME="services"
# --> LOCO_TEMPLATE= {lightweight-service|rest-api|saas}
export LOCO_APP_NAME="$1"
NEW_SERVICE_DIR="$SERVICE_DIR_NAME/$LOCO_APP_NAME"

loco new -p "./$SERVICE_DIR_NAME"
dasel put -f Cargo.toml -r toml -t string -v "$SERVICE_DIR_NAME/$LOCO_APP_NAME" '.workspace.members.append()'
dasel delete -f "./$NEW_SERVICE_DIR/Cargo.toml" workspace