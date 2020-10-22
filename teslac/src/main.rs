#[macro_use]
extern crate influx_db_client;
#[macro_use]
extern crate log;
extern crate rpassword;

use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use std::io::{stdin, stdout, Write};

use clap::{App, Arg, ArgMatches, SubCommand};
use dirs::home_dir;

use tesla::TeslaClient;

use crate::config::Config;
use crate::influx::run_influx_reporter;

mod config;
mod influx;
mod error;

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(_) => 1
    });
}

fn run() -> Result<(), ()> {
    let matches = App::new("Tesla Control")
        .version("0.2.0")
        .author("Ze'ev Klapow <zklapow@gmail.com>")
        .about("A command line interface for your Tesla")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file path")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("oauth")
                .short("o")
                .long("oauth")
                .help("Performs authentication with the Tesla servers using the prompted email address and password. Returns an oauth token when successful.")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("vehicle")
                .long("vehicle")
                .short("V")
                .help("Name of vehicle to awaken")
                .global(true)
                .takes_value(true)
        )
        .subcommand(
            SubCommand::with_name("wake")
                .about("wake up the specified vehicle")
                .arg(
                    Arg::with_name("await")
                        .help("Wait for vehicle to awaken")
                        .long("await")
                        .short("a")
                        .takes_value(false)
                )
                .arg(
                    Arg::with_name("poll-interval")
                        .help("How quickly to poll the vehicle (in seconds)")
                        .long("poll-interval")
                        .short("p")
                        .takes_value(true)
                        .default_value("5")
                )
        )
        .subcommand(
            SubCommand::with_name("get_all_data")
                .about("get all the data for the specified vehicle")
        )
        .subcommand(
            SubCommand::with_name("flash_lights")
                .about("flash lights for the specified vehicle")
        )
        .subcommand(
            SubCommand::with_name("door_unlock")
                .about("unlock the doors for the specified vehicle")
        )
        .subcommand(
            SubCommand::with_name("door_lock")
                .about("lock the doors for the specified vehicle")
        )
        .subcommand(
            SubCommand::with_name("influx")
                .about("Start the influxdb reporter")
                .arg(
                    Arg::with_name("daemon")
                        .help("Daemonize the reporter process")
                        .long("daemon")
                        .short("d")
                        .takes_value(false)
                )
        )
        .get_matches();

    if matches.is_present("oauth") {
        let mut email = String::new();
        print!("Please enter your email: ");
        let _ = stdout().flush();
        stdin().read_line(&mut email).expect("Did not enter a correct string");
        email = email.replace("\n", "").replace("\r", "");

        let password = rpassword::prompt_password_stdout("Password: ").unwrap();
        let token = TeslaClient::authenticate(email.as_str(), password.as_str());
        return if token.is_ok() {
            println!("Your token is: {}", token.unwrap());
            Ok(())
        } else {
            println!("Token error: {}", token.err().unwrap());
            Err(())
        }
    }

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
    let vehicle_name = vehicle_name.unwrap();

    if let Some(submatches) = matches.subcommand_matches("wake") {
        cmd_wake(submatches, vehicle_name, client.clone());
    } else if let Some(_submatches) = matches.subcommand_matches("get_all_data") {
        get_all_data(vehicle_name, client.clone());
    } else if let Some(_submatches) = matches.subcommand_matches("flash_lights") {
        flash_lights(vehicle_name, client.clone());
    } else if let Some(_submatches) = matches.subcommand_matches("door_unlock") {
        door_unlock(vehicle_name, client.clone());
    } else if let Some(_submatches) = matches.subcommand_matches("door_lock") {
        door_lock(vehicle_name, client.clone());
    } else if let Some(_submatches) = matches.subcommand_matches("influx") {
        if cfg.influx.is_none() {
            error!("No influx configuration present, cannot start influx reporter!");
            return Err(());
        }

        if let Err(e) = run_influx_reporter(cfg.influx.unwrap(), vehicle_name, client.clone()) {
            error!("Error in influx reporter: {}", e);
            exit(1);
        }
    } else {
        println!("No command specified")
    }

    Ok(())
}

fn cmd_wake(matches: &ArgMatches, name: String, client: TeslaClient) {
    if let Some(vehicle) = client.get_vehicle_by_name(name.as_str()).expect("Could not load vehicles") {
        let vclient = client.vehicle(vehicle.id);
        info!("Waking up");
        match vclient.wake_up() {
            Ok(_) => info!("Sent wakeup command to {}", name),
            Err(e) => error!("Wake up failed {:?}", e)
        }

        if matches.is_present("await") {
            info!("Waiting for {} to wake up.", name);
            let sleep_dur_s = Duration::from_secs(
                matches.value_of("poll-interval").unwrap().parse::<u64>()
                    .expect("Could not parse poll interval")
            );

            loop {
                if let Some(vehicle) = vclient.get().ok() {
                    if vehicle.state == "online" {
                        break;
                    } else {
                        debug!("{} is not yet online (current state is {}), waiting.", name, vehicle.state);
                    }
                }

                sleep(sleep_dur_s);
            }
        }
    } else {
        error!("Could not find vehicle named {}", name);
    }
}

fn get_all_data(name: String, client: TeslaClient) {
    if let Some(vehicle) = client.get_vehicle_by_name(name.as_str()).expect("Could not load vehicles") {
        let vclient = client.vehicle(vehicle.id);
        info!("getting all data");
        match vclient.get_all_data() {
            Ok(data) => info!("{:#?}", data),
            Err(e) => error!("get data failed {:?}", e)
        }
    } else {
        error!("Could not find vehicle named {}", name);
    }
}

fn flash_lights(name: String, client: TeslaClient) {
    if let Some(vehicle) = client.get_vehicle_by_name(name.as_str()).expect("Could not load vehicles") {
        let vclient = client.vehicle(vehicle.id);
        info!("flashing lights");
        match vclient.flash_lights() {
            Ok(_) => info!("Success"),
            Err(e) => error!("flashing lights failed {:?}", e)
        }
    } else {
        error!("Could not find vehicle named {}", name);
    }
}

fn door_unlock(name: String, client: TeslaClient) {
    if let Some(vehicle) = client.get_vehicle_by_name(name.as_str()).expect("Could not load vehicles") {
        let vclient = client.vehicle(vehicle.id);
        info!("unlocking doors");
        match vclient.door_unlock() {
            Ok(_) => info!("Success"),
            Err(e) => error!("unlocking doors failed {:?}", e)
        }
    } else {
        error!("Could not find vehicle named {}", name);
    }
}

fn door_lock(name: String, client: TeslaClient) {
    if let Some(vehicle) = client.get_vehicle_by_name(name.as_str()).expect("Could not load vehicles") {
        let vclient = client.vehicle(vehicle.id);
        info!("locking doors");
        match vclient.door_lock() {
            Ok(_) => info!("Success"),
            Err(e) => error!("locking doors failed {:?}", e)
        }
    } else {
        error!("Could not find vehicle named {}", name);
    }
}
