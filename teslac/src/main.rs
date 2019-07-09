use std::fs;
use std::path::PathBuf;

use clap::{App, Arg, ArgMatches, SubCommand};
use dirs::home_dir;
use serde::Deserialize;

use tesla::{TeslaClient, Vehicle, VehicleClient};
use tesla::reqwest;
use flexi_logger::LogSpecification;

#[macro_use]
extern crate log;

#[derive(Debug, Deserialize)]
struct Config {
    global: GlobalConfig,
    influx: Option<InfluxConfig>,
}

#[derive(Debug, Deserialize)]
struct GlobalConfig {
    api_token: String,
    default_vehicle: Option<String>,
    logspec: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InfluxConfig {
    enabled: bool,
    url: Option<String>,
    user: Option<String>,
    password: Option<String>,
}

fn main() {


    std::process::exit(match run() {
        Ok(_) => 0,
        Err(_) => 1
    });
}

fn run() -> Result<(), ()> {
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
        .arg(
            Arg::with_name("vehicle")
                .long("vehicle")
                .short("V")
                .help("Name of vehicle to awaken")
                .takes_value(true)
        )
        .subcommand(
            SubCommand::with_name("wake")
                .about("wake up the specified vehicle")
        )
        .subcommand(
            SubCommand::with_name("influx")
                .about("Start the influxdb reporter")
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

    flexi_logger::Logger::with_env_or_str(cfg.global.logspec.unwrap_or("".to_owned()))
        .format(flexi_logger::colored_with_thread)
        .start()
        .unwrap();

    let vehicle_name = matches.value_of("vehicle")
        .map(|s| s.to_owned())
        .or(cfg.global.default_vehicle);

    if vehicle_name.is_none() {
        error!("No default vehicle and no vehicle specified, aborting.");
        return Err(());
    }

    if let Some(submatches) = matches.subcommand_matches("wake") {
        cmd_wake(submatches, vehicle_name.unwrap(), client.clone());
    }

    Ok(())
}

fn cmd_wake(matches: &ArgMatches, name: String, client: TeslaClient) {
    if let Some(vehicle) = find_vehicle_by_name(&client, name.as_str()).expect("Could not load vehicles") {
        let vclient = client.vehicle(vehicle.id);
        info!("Waking up");
        match vclient.wake_up() {
            Ok(_) => info!("Sent wakeup command to {}", name),
            Err(e) => error!("Wake up failed {:?}", e)
        }
    } else {
        error!("Could not find vehicle named {}", name);
    }
}

fn find_vehicle_by_name(client: &TeslaClient, name: &str) -> Result<Option<Vehicle>, reqwest::Error> {
    let vehicle = client.get_vehicles()?.into_iter()
        .find(|v| v.display_name.to_lowercase() == name.to_lowercase());

    Ok(vehicle)
}