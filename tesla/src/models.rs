
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Vehicle {
    id: u64,
    vin: String,
    display_name: String
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