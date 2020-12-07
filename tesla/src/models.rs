use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleResponse {
    pub result: bool,
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
    pub charge_limit_soc: u32,
    pub charge_port_door_open: bool,
    pub charge_port_latch: String,
    pub charge_rate: f64,
    pub charger_actual_current: u32,
    pub charger_power: u32,
    pub charger_voltage: u32,
    pub charging_state: String,
    pub est_battery_range: f64,
    pub ideal_battery_range: f64,
    pub minutes_to_full_charge: u32,
    pub usable_battery_level: u32,
    pub time_to_full_charge: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VehicleState {
    pub api_version: i16,
    pub odometer: f64,
    pub sentry_mode: bool,
    pub locked: bool,
    pub car_version: String,
    // doors
    pub df: u8,
    pub dr: u8,
    pub pf: u8,
    pub pr: u8,
    // windows
    pub fd_window: u8,
    pub fp_window: u8,
    pub rd_window: u8,
    pub rp_window: u8,
    // front and rear trunk
    pub ft: u8,
    pub rt: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VehicleConfig {
    pub car_type: String,
    pub exterior_color: String,
    pub wheel_type: String,
    pub trim_badging: Option<String>, // TODO : This appears to not exist (anymore?). Consider removing it.
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DriveState {
    pub gps_as_of: u64,
    pub heading: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub power: f64,
    pub timestamp: u64,
    pub shift_state: Option<String>,
    pub speed: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClimateState {
    pub battery_heater: bool,
    pub defrost_mode: u8,
    pub driver_temp_setting: f64,
    pub inside_temp: f64,
    pub is_auto_conditioning_on: bool,
    pub is_climate_on: bool,
    pub is_front_defroster_on: bool,
    pub is_preconditioning: bool,
    pub is_rear_defroster_on: bool,
    pub outside_temp: f64,
    pub passenger_temp_setting: f64,
    pub remote_heater_control_enabled: bool,
    pub seat_heater_left: u8,
    pub seat_heater_right: u8,
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
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub created_at: i32,
    pub refresh_token: String,
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
