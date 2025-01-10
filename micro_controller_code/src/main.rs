use std::{thread::sleep, time::Duration};
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::{EspWifi,Configuration,ClientConfiguration};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use one_wire_bus_2::{OneWire, OneWireError};
use ds18b20_2::Ds18b20;


include!(concat!(env!("OUT_DIR"), "/wifi_config.rs"));


fn main() {
    // ??? 
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
    // // turn password and ssid into an actual string 
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
    let temp_pin      = PinDriver::input_output(peripherals.pins.gpio15).unwrap();

    // create 1-wire bus connection to talk to the sensor with a delay
    let mut delay = Ets;
    let mut onewire = OneWire::new(temp_pin).unwrap();

    // collect devices from an iterator into a vector 
    let devices: Vec<_> = onewire.devices(false,&mut delay).collect();
    // check that devices where actually found 
    if devices.is_empty(){
        println!("No Devices found......");
        return;
    }

    // at this stage only one device should be connected so just use the first 
    let sensor_address = devices[0].unwrap();
    println!("Found Temperature Sensor at: {:?}",sensor_address);
    // create an instance of the sensor 
    let temp_sensor = Ds18b20::new::<OneWireError<EspError>>(sensor_address).unwrap();

    // Main loop to print IP address info every 10 seconds
    loop {
        if let Ok(ip_info) = wifi_driver.sta_netif().get_ip_info() {
            println!("IP info: {:?}", ip_info);
            // get the temp from the sensoe 
            let temp_reading = temp_sensor.read_data(&mut onewire, &mut delay).unwrap();
            // Print the temperature.
            println!("Temperature: {:.2} °C", temp_reading.temperature);

        } else {
            println!("Failed to get IP info");
        }
        sleep(Duration::new(10, 0));
    }
   
}