#!/bin/bash

## ADD READING FROM BASH HERE ##
SSID=$(jq .ssid wifi_config.json)
PASSWORD=$(jq .password wifi_config.json)

echo "Compiling build ..."
cd micro_controller_code 
SSID="$SSID" PASSWORD="$PASSWORD" cargo build --target xtensa-esp32-espidf --release
echo "reseting device ..."
espflash erase-flash
echo "pushing new code ..."
espflash flash --port /dev/ttyUSB0 /home/ryan/AquaInspector/AquaInspector-Sensor/micro_controller_code/target/xtensa-esp32-espidf/release/micro_controller_code --monitor
