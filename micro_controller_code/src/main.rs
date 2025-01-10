use std::{thread::sleep, time::Duration};
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::{EspWifi,Configuration,ClientConfiguration};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use one_wire_bus::{OneWire, OneWireError, OneWireResult};
use ds18b20::Ds18b20;


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

    // Main loop 
    loop {
        if let Ok(ip_info) = wifi_driver.sta_netif().get_ip_info() {
           
            let _ = temp_sensor.start_temp_measurement(&mut onewire, &mut delay);
            // Delay to give the sensor time to measure the temperature
            sleep(Duration::from_secs(1));

            // get the temp from the sensor with error handling
            match temp_sensor.read_data(&mut onewire, &mut delay) {
                //if a temperature was read successfully
                Ok(temp_data) => {
                    println!("Temperature: {:.2} °C",&temp_data.temperature)
                }
                // handle error case
                Err(e) => {
                    eprintln!("Error reading temperature: {:?}", e);
                }                
            }
        }
        else{
            println!("Failed to get IP...");
        }
        // wait ten seconds 
        sleep(Duration::new(10, 0));
    
    }  
}