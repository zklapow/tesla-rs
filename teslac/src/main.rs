use std::fs;
use dirs::home_dir;

use tesla::TeslaClient;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    api_token: String,
}

fn main() {
    let config_path = home_dir().expect("Cannot find home dir")
        .join(".teslac");

    let config_data = fs::read_to_string(config_path).expect("Cannot read config");

    let cfg: Config = toml::from_str(config_data.as_str()).expect("Cannot parse config");

    let client = TeslaClient::default(cfg.api_token.as_str());

    let vehicles = client.get_vehicles().expect("Cannot fetch vehicles");

    println!("Vehicles: {:?}", vehicles);

    let atas = vehicles.get(0).expect("No vehicle");
    let atas_client = client.use_vehicle(atas.id);
    let soc = atas_client.get_soc();

    println!("SOC: {:?}", soc);
}

