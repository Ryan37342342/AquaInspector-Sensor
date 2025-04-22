use std::{thread::sleep, time::Duration};

use esp_idf_svc::hal::{delay::Ets,gpio::PinDriver,peripherals::Peripherals};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::{EspWifi,Configuration,ClientConfiguration};

use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use esp_idf_svc::http::client::{Configuration as HttpConfiguration,EspHttpConnection};

use embedded_svc::{
    http::Method,
    io::Write,
};

use one_wire_bus::{OneWire, OneWireError};
use ds18b20::Ds18b20;
use chrono::{DateTime, Local,Utc};
use serde::{Serialize};
use heapless::String;


include!(concat!(env!("OUT_DIR"), "/wifi_config.rs"));

#[derive(Serialize)]
struct TemperaturePayload {
    tank_number: u32,
    temp :f32,
    time_stamp: std::string::String,
}

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    // ??? 
     // Initialize logging and panic handlers
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::sys::link_patches();
    println!("Entered Main function!");
   
    // Initialize peripherals, event loop, and storage
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();     
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // Create a new WiFi driver instance
    let mut wifi_driver = EspWifi::new(
        peripherals.modem,
        sys_loop,
        Some(nvs),
    ).unwrap();
    
    // get the wifi config
    // turn password and ssid into an actual string 
     let mut ssid = heapless::String::<32>::new();
     ssid.push_str(WIFI_SSID).expect("SSID too long");

     let mut password = heapless::String::<64>::new();
     password.push_str(WIFI_PASSWORD).expect("Password too long");

    // Configure WiFi with SSID and password
    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid:ssid,       
        password:password, 
        ..Default::default()
    })).unwrap();

    // Start and connect WiFi
    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();

    // Loop until connected, printing configuration info for debugging
    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
        sleep(Duration::from_secs(1)); // Add a small delay to avoid spamming output
    }


    println!("Connected to WiFi!");

    // now we start to connect to the senors 
    // setup pin that the temperature sensor will send data down
    let temp_pin = PinDriver::input_output(peripherals.pins.gpio15).unwrap();

    // create 1-wire bus connection to talk to the sensor with a delay
    let mut delay = Ets;
    let mut onewire = OneWire::new(temp_pin).unwrap();

    // collect devices from an iterator into a vector 
    let devices: Vec<_> = onewire.devices(false,&mut delay).collect();
    // check that devices where actually found 
    if devices.is_empty(){
        println!("No Devices found......");
        return Ok(());
    }

    // at this stage only one device should be connected so just use the first 
    let sensor_address = devices[0].unwrap();
    println!("Found Temperature Sensor at: {:?}",sensor_address);
    // create an instance of the sensor 
    let temp_sensor = Ds18b20::new::<OneWireError<EspError>>(sensor_address).unwrap();

  
    // create headers out of loop
    let headers = &[("Content-Type", "application/json")];
    //create a array of len 3 to hold temperatures 
    let mut temp_array :[f32;3] = [0.0;3];

    
    // Main loop 
    loop {
    if wifi_driver.sta_netif().get_ip_info().is_err() {
            println!("Failed to get IP...");
            sleep(Duration::new(10, 0));
            continue;
        }
    for i in 0..3 {
            let _ = temp_sensor.start_temp_measurement(&mut onewire, &mut delay);
            sleep(Duration::from_secs(2));
    
            match temp_sensor.read_data(&mut onewire, &mut delay) {
                Ok(temp_data) => {
                    println!("Temperature: {:.2} °C", temp_data.temperature);
                    temp_array[i] = temp_data.temperature;
                }
                Err(e) => {
                    eprintln!("Error reading temperature: {:?}", e);
                    continue;
                }
            }
        }
    let average_reading = temp_array.iter().sum::<f32>() / temp_array.len() as f32;
    let time_now = Utc::now();
    let formatted_timestamp = time_now.to_rfc3339();
    
    let connection = EspHttpConnection::new(&HttpConfiguration::default())?;
     // create http client
     let mut client = embedded_svc::http::client::Client::wrap(connection);

    let payload = TemperaturePayload {
            temp: average_reading,
            time_stamp: formatted_timestamp,
            tank_number: TANK_NUMBER.parse().unwrap_or(u32::MAX),
        };

    let payload_json = match serde_json::to_string(&payload) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize JSON Payload: {:?}", e);
                sleep(Duration::new(10, 0));
                continue;
            }
        };
    //println!("Request Body is: {}", payload_json);
    println!("sending request to {}", TEMP_API_URL);
    let mut request =  client.request(Method::Post,TEMP_API_URL,headers)?;
  
    request.write_all(&payload_json.as_bytes())?;
    request.flush()?;
    let response = request.submit()?;
    println!("Response status: {}", response.status());
    }

    Ok(())
}