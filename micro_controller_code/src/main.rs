use std::error::Error;
use std::net::{IpAddr, Ipv4Addr};
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
use chrono::Utc;
use serde::{Serialize};



include!(concat!(env!("OUT_DIR"), "/wifi_config.rs"));

#[derive(Serialize)]
struct TemperaturePayload {
    tank_number: u32,
    temp :f32,
    time_stamp: std::string::String,
}

#[derive(Serialize)]
struct LogMessage {
    tank_number: u32,
    message_type: std::string::String,
    message: std::string::String,
    time_stamp: std::string::String,
}


fn main() -> Result<(), Box<dyn std::error::Error>>  {
    // set up api addresses 
    const TEMP_API_ENDPOINT: &str = "api/tank/temperature-reading";
    const LOG_API_ENDPOINT: &str = "api/tank/log";
    let temp_url = format!("{}{}",BASE_URL,TEMP_API_ENDPOINT);
    let logging_url = format!("{}{}",BASE_URL,LOG_API_ENDPOINT);

    // read in enviroment varaiable 
    let tank_number =TANK_NUMBER.parse().unwrap_or(u32::MAX);

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

    sleep(Duration::from_secs(3));
    println!("logging url: {}",&logging_url);
    let _logging_result = match log_message(tank_number, "INFO", "CONNECTED TO WIFI!", &logging_url){
        Ok(()) => {
            println!("Logging is working!");
           
        },
        Err(e)=> {
            println!("Error occured when logging message: {}", e)
        }
    };
    // now we start to connect to the senors 
    // setup pin that the temperature sensor will send data down
    let temp_pin = PinDriver::input_output(peripherals.pins.gpio15).unwrap();

    // create 1-wire bus connection to talk to the sensor with a delay
    let mut delay = Ets;
    let mut onewire = OneWire::new(temp_pin).unwrap();

    // loop to connect to sensors as sometimes this takes a few goes
    let sensor_address = loop {
        // try to find devices and get first address
        let devices: Vec<_> = onewire.devices(false, &mut delay).collect();

        if devices.is_empty() {
            eprintln!("No devices found, retrying in 5 seconds...");
            sleep(Duration::from_secs(5));
            continue;  // retry loop
        }

        match devices.get(0) {
            Some(Ok(addr)) => break *addr,  // success! break loop returning addr
            Some(Err(e)) => {
                eprintln!("Error getting sensor address: {:?}, retrying in 5 seconds...", e);
                sleep(Duration::from_secs(5));
                continue;
            }
            None => {
                eprintln!("No devices found at all, retrying in 5 seconds...");
                sleep(Duration::from_secs(5));
                continue;
            }
        }
    };
   
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
    else {
        let ip_info = wifi_driver.sta_netif().get_ip_info().unwrap();
        println!("IP is: {}",ip_info.ip)
    }
    
    for i in 0..3 {
            let _ = temp_sensor.start_temp_measurement(&mut onewire, &mut delay);
            sleep(Duration::from_secs(20));
    
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

    ensure_wifi_connected(&mut wifi_driver)?;
    let average_reading = temp_array.iter().sum::<f32>() / temp_array.len() as f32;
    let time_now = Utc::now();
    let formatted_timestamp = time_now.to_rfc3339();
    
    let connection = EspHttpConnection::new(&HttpConfiguration::default())?;
     // create http client
     let mut client = embedded_svc::http::client::Client::wrap(connection);

    let payload = TemperaturePayload {
            temp: average_reading,
            time_stamp: formatted_timestamp,
            tank_number: tank_number,
        };

    let payload_json = match serde_json::to_string(&payload) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize JSON Payload: {:?}", e);
                sleep(Duration::new(10, 0));
                continue;
            }
        };
    ensure_wifi_connected(&mut wifi_driver)?;
    //println!("Request Body is: {}", payload_json);
    println!("sending request to {}", temp_url);
    let mut request =  client.request(Method::Post,&temp_url,headers)?;
  
    request.write_all(&payload_json.as_bytes())?;
    request.flush()?;
    let response = request.submit()?;
    println!("Response status: {}", response.status());
    }

    Ok(())
}

pub fn log_message(tank_number: u32, message_type: &str, message: &str, log_url: &str) -> Result<(), Box<dyn Error>> {
    let time_now = Utc::now();
    let formatted_timestamp = time_now.to_rfc3339();

    let log_entry = LogMessage {
        tank_number,
        message_type: message_type.to_string(),
        message: message.to_string(),
        time_stamp: formatted_timestamp,
    };

    let json_payload = serde_json::to_string(&log_entry)?; // uses `?` to return error
    
    // 🖨️ Debug print for the URL and payload
    println!("Sending log to URL: {}", log_url);
    println!("Log JSON Payload: {}", json_payload);
    
    // create the connection
    let connection = EspHttpConnection::new(&HttpConfiguration::default())?;
    let mut client = embedded_svc::http::client::Client::wrap(connection);
    let headers = &[("Content-Type", "application/json")];

    let mut request = client.request(Method::Post, log_url, headers)?;
    request.write_all(&json_payload.as_bytes())?;
    request.flush()?;
    let response = request.submit()?;
    println!("Log sent. Response status: {}", response.status());

    Ok(())
}

fn ensure_wifi_connected(wifi: &mut EspWifi) -> Result<(), Box<dyn std::error::Error>> {
    let mut retries = 50;

    while retries > 0 {
        if wifi.is_connected().unwrap_or(false) {
            let ip_info = wifi.ap_netif().get_ip_info()?;
            println!("Passed Check: WiFi connected.");
            return Ok(());
        } else {
            println!("WiFi disconnected. Attempting to reconnect...");
            wifi.connect()?;
            std::thread::sleep(std::time::Duration::from_secs(3));
            retries -= 1;
        }
    }

    Err("Failed to reconnect to WiFi after retries".into())
}