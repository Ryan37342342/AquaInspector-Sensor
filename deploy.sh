#!/bin/bash
echo "Compiling build ..."
cd micro_controller_code 
cargo build --target xtensa-esp32-espidf --release
echo "reseting device ..."
espflash erase-flash
echo "pushing new code ..."
espflash flash --port /dev/ttyUSB0 /home/ryan/AquaInspector/AquaInspector-Sensor/micro_controller_code/target/xtensa-esp32-espidf/release/micro_controller_code \
--monitor
