
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleResponse {
    pub result: boolc,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vehicle {
    pub id: u64,
    pub vehicle_id: u64,
    pub vin: String,
    pub display_name: String,
    pub state: String,
    pub id_s: String,
    pub tokens: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateOfCharge {
    pub battery_heater_on: bool,
    pub battery_level: u32,
    pub battery_range: f64,
    pub charge_current_request: u32,
    pub charge_current_request_max: u32,
    pub charger_power: u32,
    pub charger_voltage: u32,
    pub charging_state: String,
    pub est_battery_range: f64,
    pub ideal_battery_range: f64,
    pub usable_battery_level: u32,
    pub time_to_full_charge: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VehicleState {
    pub odometer: f64,
    pub sentry_mode: bool,
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VehicleConfig {
    pub car_type: String,
    pub exterior_color: String,
    pub wheel_type: String,
    pub trim_badging: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DriveState {
    pub gps_as_of: u64,
    pub heading: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub power: f64,
    pub timestamp: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClimateState {
    pub inside_temp: f64,
    pub outside_temp: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FullVehicleData {
    pub id: u64,
    pub user_id: u64,
    pub vehicle_id: u64,
    pub state: String,
    pub charge_state: StateOfCharge,
    pub vehicle_state: VehicleState,
    pub drive_state: DriveState,
    pub climate_state: ClimateState,
    pub gui_settings: GuiSettings,
    pub vehicle_config: VehicleConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuiSettings {
    pub gui_charge_rate_units: String,
    pub gui_distance_units: String,
    pub gui_temperature_units: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseArray<T> {
    response: Vec<T>,
    count: u32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<T> {
    response: T,
}

impl <T> ResponseArray<T> {
    pub fn into_response(self) -> Vec<T> {
        self.response
    }
}

impl <T> Response<T> {
    pub fn into_response(self) -> T {
        self.response
    }
}
