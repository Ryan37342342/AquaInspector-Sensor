use std::{thread::sleep, time::Duration};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::wifi::{EspWifi,Configuration,ClientConfiguration};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;


// struct to hold the wifi connection details 
#[derive(Deserialize)]
struct WifiConfig{
    ssid: String,
    password : String,
}

// function to read wifi details from file 
fn read_wifi_config()->WifiConfig{
    //open the file 
    let mut file = File::open("/spiffs/wifi_config.json").expect("wifi config file should exist on device");
    let mut buffer = String::new();
    //read the file into a json string                                                   
    file.read_to_string(&mut buffer).expect("Expected to be able to read file");
    // turn json string into wifiConfig 
    let wifi_config : WifiConfig = serde_json::from_str(&buffer).expect("Expected Json String");
    return wifi_config;
}
fn main() {
    // Required for ESP32 Rust compatibility.    
    esp_idf_svc::sys::link_patches();
    println!("Entered Main function!");

    // Initialize peripherals, event loop, and NVS (Non-Volatile Storage)
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();     
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // Create a new WiFi driver instance
    let mut wifi_driver = EspWifi::new(
        peripherals.modem,
        sys_loop,
        Some(nvs),
    ).unwrap();


    // Configure WiFi with your SSID and password
    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid:"".try_into().unwrap(),        // Update with your WiFi SSID
        password:"".try_into().unwrap(), // Update with your WiFi Password
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

    // Main loop to print IP address info every 10 seconds
    loop {
        if let Ok(ip_info) = wifi_driver.sta_netif().get_ip_info() {
            println!("IP info: {:?}", ip_info);
        } else {
            println!("Failed to get IP info");
        }
        sleep(Duration::new(10, 0));
    }
}