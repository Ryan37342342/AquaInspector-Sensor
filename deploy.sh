#!/bin/bash

## ADD READING FROM BASH HERE ##
SSID=$(jq .ssid wifi_config.json)
PASSWORD=$(jq .password wifi_config.json)
TEMP_API_URL=$(jq .temp_api_url wifi_config.json)

set -e

echo "Compiling build ..."
cd micro_controller_code 
cargo clean
SSID="$SSID" PASSWORD="$PASSWORD" TEMP_API_URL="$TEMP_API_URL" cargo build --target xtensa-esp32-espidf --release
echo "reseting device ..."
espflash erase-flash
echo "pushing new code ..."
espflash flash --port /dev/ttyUSB0 /home/ryan/AquaInspector/AquaInspector-Sensor/micro_controller_code/target/xtensa-esp32-espidf/release/micro_controller_code --monitor
