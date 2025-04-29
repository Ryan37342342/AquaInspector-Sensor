# AquaInspector-Sensor
This is code for the microcontroller and sensor setup, once complete this will send data to the AquaInspector API and Database for later use

## Current Functionality 
 - Expects a wifi_config.json file with the following string fields ssid and password, these should be for an available network
 - wifi_config.json should also contain a tank_number variable, this should be unique between all tanks in the system
 - Bash script will take and parse this into the rust binary file using the build-script option so it can be accessed 
 - Temperature is read from the sensor and errors are handled gracefully
 - Temperatures are now send and read into DB via http and the central API
 - Error Messages are now logged in the database 


## ToDo :
 - Investigate other sensors
