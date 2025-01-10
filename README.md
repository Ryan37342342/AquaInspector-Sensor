# AquaInspector-Sensor
This is code for the microcontroller and sensor setup, once complete this will send data to the AquaInspector API and Database for later use

## Current Functionality 
 - Expects a wifi_config.json file with the following string fields ssid and password, these should be for an available network
 - Bash script will take and parse this into the rust binary file using the build-script option so it can be accessed 
 - Temperature is read from the sensor and errors are handled gracefully

## ToDo :
 - setup http api to return temp reading to central API
