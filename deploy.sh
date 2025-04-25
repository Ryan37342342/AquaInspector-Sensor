#!/bin/bash

## ADD READING FROM BASH HERE ##
SSID=$(jq .ssid wifi_config.json)
PASSWORD=$(jq .password wifi_config.json)
BASE_URL=$(jq .base_url wifi_config.json)
TANK_NUMBER=$(jq .tank_number wifi_config.json)

set -e

echo "Compiling build ..."
cd micro_controller_code 
# uncomment to clean up packages if broken install 
#cargo clean
cargo update 
export RUST_BACKTRACE=1
RUST_LOG=debug SSID="$SSID" PASSWORD="$PASSWORD" BASE_URL="$BASE_URL" TANK_NUMBER="$TANK_NUMBER" cargo build --target xtensa-esp32-espidf --release
echo "reseting device ..."
espflash erase-flash
echo "pushing new code ..."
espflash flash --port /dev/ttyUSB0 /home/ryan/AquaInspector/AquaInspector-Sensor/micro_controller_code/target/xtensa-esp32-espidf/release/micro_controller_code --monitor
