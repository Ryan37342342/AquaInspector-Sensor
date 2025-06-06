use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Default setup generated by template
    embuild::espidf::sysenv::output();

    // Retrieve required environment variables NOTE these are passed in the the bash script 
    let ssid = env::var("SSID").expect("Environment variable `SSID` not provided");
    let password = env::var("PASSWORD").expect("Environment variable `PASSWORD` not provided");
    let base_url = env::var("BASE_URL").expect("Environment variable `BASE_URL` not provided");
    let tank_number = env::var("TANK_NUMBER").expect("Enviroment variable `TANK_NUMBER` not provided");

    // Generate a file in the build directory to be included in the binary
    let out_dir = env::var("OUT_DIR").expect("Failed to retrieve OUT_DIR environment variable");
    let dest_path = Path::new(&out_dir).join("wifi_config.rs");

    // Write configuration to the file
    fs::write(
        &dest_path,
        format!(
            r#"
            pub const WIFI_SSID: &str = {ssid};
            pub const WIFI_PASSWORD: &str = {password};
            pub const BASE_URL: &str = {base_url};
            pub const TANK_NUMBER: &str ={tank_number};
            "#,
            ssid = ssid,
            password = password,
            base_url = base_url,
        ),
    )
    .expect("Failed to write wifi_config.rs");

    // Log the generated file path for debugging purposes
    println!("Generated configuration file: {}", dest_path.display());

    // Tell Cargo to rerun the build script if these environment variables change
    println!("cargo:rerun-if-env-changed=SSID");
    println!("cargo:rerun-if-env-changed=PASSWORD");
    println!("cargo:rerun-if-env-changed=BASE_URL");
}