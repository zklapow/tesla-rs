use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use serde_json;

use regex::Regex;

use tesla::*;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4321").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let http_content = std::str::from_utf8(&buffer).unwrap();
    let re: Regex = Regex::new(r"^(\w+)\s+(.+)\s+HTTP").unwrap();
    let regex_captures = re.captures(http_content).unwrap();
    let method = regex_captures.get(1).unwrap().as_str();
    let url = regex_captures.get(2).unwrap().as_str();

    println!("method '{}', url '{}'", method, url);

    let mut status_line = "HTTP/1.1 200 OK";
    let mut contents = String::new();
    if method == "POST" && Regex::new(r"^/oauth/token$").unwrap().is_match(url) {
        let auth_response = AuthResponse {
            access_token: "magic_token_for_api_calls".to_string(),
            token_type: "bearer".to_string(),
            expires_in: 3888000,
            created_at: 1571519135,
            refresh_token: "".to_string()
        };
        contents = serde_json::to_string(&auth_response).unwrap();
    } else if method == "GET" && Regex::new(r"^/api/\d+/vehicles$").unwrap().is_match(url) {
        let vehicle = Vehicle {
            id: 0,
            vehicle_id: 0,
            vin: "ABC1234567890".to_string(),
            display_name: "TEST CAR".to_string(),
            state: "".to_string(),
            id_s: "".to_string(),
            tokens: vec![]
        };
        let vehicles = vec![vehicle];
        contents = format!("{{ \"response\" : {}, \"count\": {} }}", serde_json::to_string(&vehicles).unwrap(), vehicles.len());
    } else if method == "GET" && Regex::new(r"^/api/\d+/vehicles/\d+/vehicle_data$").unwrap().is_match(url) {
        let full_vehicle_data = FullVehicleData {
            id: 0,
            user_id: 0,
            vehicle_id: 0,
            state: "Driving".to_string(),
            charge_state: StateOfCharge {
                battery_heater_on: false,
                battery_level: 0,
                battery_range: 0.0,
                charge_current_request: 0,
                charge_current_request_max: 0,
                charge_limit_soc: 0,
                charge_port_door_open: false,
                charge_port_latch: "".to_string(),
                charge_rate: 0.0,
                charger_actual_current: 0,
                charger_power: 0,
                charger_voltage: 0,
                charging_state: "".to_string(),
                est_battery_range: 0.0,
                ideal_battery_range: 0.0,
                minutes_to_full_charge: 0,
                usable_battery_level: 0,
                time_to_full_charge: 0.0
            },
            vehicle_state: VehicleState {
                api_version: 0,
                odometer: 0.0,
                sentry_mode: false,
                locked: false,
                car_version: "".to_string(),
                df: 0,
                dr: 0,
                pf: 0,
                pr: 0,
                fd_window: 0,
                fp_window: 0,
                rd_window: 0,
                rp_window: 0,
                ft: 0,
                rt: 0
            },
            drive_state: DriveState {
                gps_as_of: 0,
                heading: 0,
                latitude: 0.0,
                longitude: 0.0,
                power: 0.0,
                timestamp: 0,
                shift_state: None,
                speed: None
            },
            climate_state: ClimateState {
                battery_heater: false,
                defrost_mode: 0,
                driver_temp_setting: 0.0,
                inside_temp: 0.0,
                is_auto_conditioning_on: false,
                is_climate_on: false,
                is_front_defroster_on: false,
                is_preconditioning: false,
                is_rear_defroster_on: false,
                outside_temp: 0.0,
                passenger_temp_setting: 0.0,
                remote_heater_control_enabled: false,
                seat_heater_left: 0,
                seat_heater_right: 0
            },
            gui_settings: GuiSettings {
                gui_charge_rate_units: "".to_string(),
                gui_distance_units: "".to_string(),
                gui_temperature_units: "".to_string()
            },
            vehicle_config: VehicleConfig {
                car_type: "".to_string(),
                exterior_color: "".to_string(),
                wheel_type: "".to_string(),
                trim_badging: None
            }
        };
        contents = format!("{{ \"response\" : {} }}", serde_json::to_string(&full_vehicle_data).unwrap());
    } else {
        println!("No route found for {}", http_content);
        status_line = "HTTP/1.1 404 NOT FOUND";
    };

    let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", status_line, contents.len(), contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
