use std::fs;

use clap::{App, Arg, SubCommand, ArgMatches};
use dirs::home_dir;
use serde::Deserialize;

use tesla::{TeslaClient, VehicleClient, Vehicle};
use tesla::reqwest;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Config {
    global: GlobalConfig,
}

#[derive(Debug, Deserialize)]
struct GlobalConfig {
    api_token: String,
}

fn main() {
    let matches = App::new("Tesla Control")
        .version("0.1.0")
        .author("Ze'ev Klapow <zklapow@gmail.com>")
        .about("A command line interface for your tesla")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file path")
                .takes_value(true)
        )
        .subcommand(
            SubCommand::with_name("wake")
                .about("wake up the specified vehicle")
                .arg(
                    Arg::with_name("vehicle")
                        .help("Name of vehicle to awaken")
                        .required(true)
                        .index(1)
                )
        )
        .get_matches();

    let config_path_default = home_dir()
        .unwrap_or(PathBuf::from("/"))
        .join(".teslac");

    let config_path = matches.value_of("config")
        .map(|p| PathBuf::from(p))
        .unwrap_or(config_path_default);

    let config_data = fs::read_to_string(config_path).expect("Cannot read config");
    let cfg: Config = toml::from_str(config_data.as_str()).expect("Cannot parse config");
    let client = TeslaClient::default(cfg.global.api_token.as_str());

    if let Some(submatches) = matches.subcommand_matches("wake") {
        cmd_wake(submatches, client.clone());
    }

//    let vehicles = client.get_vehicles().expect("Cannot fetch vehicles");
//
//    println!("Vehicles: {:?}", vehicles);
//
//    let atas = vehicles.get(0).expect("No vehicle");
//    let atas_client: VehicleClient = client.vehicle(atas.id);
//
//    let atas = atas_client.get();
//    println!("Vehicle state: {:?}", atas);
//
////    if atas.state.to_lowercase() == "offline" {
////        println!("Waking vehicle");
////        atas_client.wake_up();
////    }
//
//    let data = atas_client.get_all_data();
//
//    println!("Data: {:?}", data);
}

fn cmd_wake(matches: &ArgMatches, client: TeslaClient) {
    // This arg is required, ok to unwrap
    let name = matches.value_of("vehicle").unwrap();

    if let Some(vehicle) = find_vehicle_by_name(&client, name).expect("Could not load vehicles") {
        let vclient = client.vehicle(vehicle.id);
        println!("Waking up");
        match vclient.wake_up() {
            Ok(_) => println!("Sent wakeup command to {}", name),
            Err(e) => println!("Wake up failed {:?}", e)
        }
    } else {
        println!("Could not find vehicle named {}", name);
    }
}

fn find_vehicle_by_name(client: &TeslaClient, name: &str) -> Result<Option<Vehicle>, reqwest::Error> {
    let vehicle = client.get_vehicles()?.into_iter()
        .find(|v| v.display_name.to_lowercase() == name.to_lowercase());

    Ok(vehicle)
}