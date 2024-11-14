use std::{thread::sleep, time::Duration};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::wifi::{EspWifi,Configuration,ClientConfiguration};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use heapless::String;


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

    let mut ssid: String<32> = String::from("");
    let mut password: String<64> = String :: from("");

    // Configure WiFi with your SSID and password
    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid,        // Update with your WiFi SSID
        password, // Update with your WiFi Password
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