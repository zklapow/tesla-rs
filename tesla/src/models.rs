
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Vehicle {
    pub id: u64,
    pub vehicle_id: u64,
    pub vin: String,
    pub display_name: String,
    pub state: String,
    pub id_s: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateOfCharge {
    pub battery_level: u32,
    pub battery_range: f32,
    pub charge_current_request: u32,
    pub charge_current_request_max: u32,
    pub ideal_battery_range: f32,
    pub usable_battery_level: u32
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