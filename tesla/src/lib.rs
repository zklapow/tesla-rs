use std::collections::HashMap;

use serde::{Serialize, Deserialize};

pub use reqwest;
use reqwest::blocking::Client;
use reqwest::header;

pub use models::*;

mod models;

const DEFAULT_BASE_URI: &str = "https://owner-api.teslamotors.com/api/1/";
const ENDPOINT_GET_VEHICLES: &str = "vehicles";
#[allow(dead_code)]
const ENDPOINT_GET_VEHICLE: &str = "vehicles/{}";

const VEHICLE_CHARGE_STATE: &str = "data_request/charge_state";
const VEHICLE_GUI_SETTINGS: &str = "data_request/gui_settings";
const VEHICLE_DATA: &str = "vehicle_data";

const VEHICLE_COMMAND_WAKE: &str = "wake_up";
const VEHICLE_COMMAND_FLASH: &str = "flash_lights";
const VEHICLE_COMMAND_DOOR_UNLOCK: &str = "door_unlock";
const VEHICLE_COMMAND_DOOR_LOCK: &str = "door_lock";
const VEHICLE_COMMAND_HONK_HORN: &str = "honk_horn";
const VEHICLE_COMMAND_AUTO_CONDITIONING_START: &str = "auto_conditioning_start";
const VEHICLE_COMMAND_AUTO_CONDITIONING_STOP: &str = "auto_conditioning_stop";
const VEHICLE_COMMAND_REMOTE_START_DRIVE: &str = "remote_start_drive";
const VEHICLE_COMMAND_CHARGE_PORT_DOOR_OPEN: &str = "charge_port_door_open";
const VEHICLE_COMMAND_CHARGE_PORT_DOOR_CLOSE: &str = "charge_port_door_close";

// We expect here because this is parsing a const and will not fail
macro_rules! endpoint_url {
    ($client: ident, $e:expr) => {
        $client.get_base_url().join($e).expect("cannot parse endpoint")
    }
}

#[derive(Clone)]
pub struct TeslaClient {
    pub api_root: reqwest::Url,
    client: Client,
}

#[derive(Clone)]
pub struct VehicleClient {
    tesla_client: TeslaClient,
    vehicle_id: u64,
}

impl TeslaClient {
    pub fn authenticate(email: String, password: String) -> Result<String, String> {
        let root_url = reqwest::Url::parse(DEFAULT_BASE_URI).expect("Could not parse API root");
        let url = root_url.join("/oauth/token").expect("Could not parse API endpoint");
        let mut map = HashMap::new();
        map.insert("grant_type", "password");
        // Use client_id and client_secret obtained from Android/iOS app. These are not the user's api key.
        map.insert("client_id", "81527cff06843c8634fdc09e8ac0abefb46ac849f38fe1e431c2ef2106796384");
        map.insert("client_secret", "c7257eb71a564034f9419ee651c7d0e5f7aa6bfbd18bafb5c5c033b093bb2fa3");
        map.insert("email", email.as_str());
        map.insert("password", password.as_str());
        let client = Client::new();

        #[derive(Serialize, Deserialize, Debug)]
        struct AuthResponse {
            response: Option<String>,
            access_token: Option<String>,
            token_type: Option<String>,
            expires_in: Option<i32>,
            created_at: Option<i32>,
            refresh_token: Option<String>,
        }

        let resp: AuthResponse = client.post(url)
            .json(&map)
            .send().unwrap()
            .json().unwrap();
        return if resp.access_token.is_some() {
            let expires_in = resp.expires_in.unwrap();
            let expires_in_days = expires_in / 60 / 60 / 24;
            println!("The access token will expire in {} days", expires_in_days);
            Ok(resp.access_token.unwrap())
        } else {
            let error_response = resp.response.unwrap_or(String::from("unknown reason"));
            Err(format!("Did not get an access token because of {}", error_response))
        }
    }

    pub fn default(access_token: &str) -> TeslaClient {
        TeslaClient::new(DEFAULT_BASE_URI, access_token)
    }

    pub fn new(api_root: &str, access_token: &str) -> TeslaClient {
        let mut headers = header::HeaderMap::new();

        let auth_value = header::HeaderValue::from_str(format!("Bearer {}", access_token).as_str()).expect("bearer header");

        headers.insert(header::AUTHORIZATION, auth_value);

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Could not create client");

        TeslaClient {
            api_root: reqwest::Url::parse(api_root).expect("Could not parse API root"),
            client,
        }
    }

    pub fn vehicle(&self, vehicle_id: u64) -> VehicleClient {
        VehicleClient {
            tesla_client: self.clone(),
            vehicle_id,
        }
    }

    pub fn get_vehicles(&self) -> Result<Vec<Vehicle>, reqwest::Error> {
        let url = endpoint_url!(self, ENDPOINT_GET_VEHICLES);

        let vehicle_response: ResponseArray<Vehicle> = self.client.get(url)
            .send()?
            .json()?;

        Ok(vehicle_response.into_response())
    }

    pub fn get_vehicle_by_name(&self, name: &str) -> Result<Option<Vehicle>, reqwest::Error> {
        let vehicle = self.get_vehicles()?.into_iter()
            .find(|v| v.display_name.to_lowercase() == name.to_lowercase());

        Ok(vehicle)
    }

    fn get_base_url(&self) -> reqwest::Url {
        self.api_root.clone()
    }
}

impl VehicleClient {
    pub fn wake_up(&self) -> Result<Vehicle, reqwest::Error> {
        let url = endpoint_url!(self, VEHICLE_COMMAND_WAKE);

        let resp: Response<Vehicle> = self.tesla_client.client.post(url)
            .send()?
            .json()?;

        Ok(resp.into_response())
    }

    pub fn flash_lights(&self) -> Result<SimpleResponse, reqwest::Error> {
        self.post_simple_command(VEHICLE_COMMAND_FLASH)
    }

    pub fn door_unlock(&self) -> Result<SimpleResponse, reqwest::Error> {
        self.post_simple_command(VEHICLE_COMMAND_DOOR_UNLOCK)
    }

    pub fn door_lock(&self) -> Result<SimpleResponse, reqwest::Error> {
        self.post_simple_command(VEHICLE_COMMAND_DOOR_LOCK)
    }

    pub fn honk_horn(&self) -> Result<SimpleResponse, reqwest::Error> {
        self.post_simple_command(VEHICLE_COMMAND_HONK_HORN)
    }

    pub fn auto_conditioning_start(&self) -> Result<SimpleResponse, reqwest::Error> {
        self.post_simple_command(VEHICLE_COMMAND_AUTO_CONDITIONING_START)
    }

    pub fn auto_conditioning_stop(&self) -> Result<SimpleResponse, reqwest::Error> {
        self.post_simple_command(VEHICLE_COMMAND_AUTO_CONDITIONING_STOP)
    }

    pub fn remote_start_drive(&self) -> Result<SimpleResponse, reqwest::Error> {
        // TODO : Need to pass the password in the querystring
        let url = self.get_command_url(VEHICLE_COMMAND_REMOTE_START_DRIVE);
        let resp: Response<SimpleResponse> = self.tesla_client.client.post(url)
            .send()?
            .json()?;
        Ok(resp.into_response())
    }

    pub fn charge_port_door_open(&self) -> Result<SimpleResponse, reqwest::Error> {
        self.post_simple_command(VEHICLE_COMMAND_CHARGE_PORT_DOOR_OPEN)
    }

    pub fn charge_port_door_close(&self) -> Result<SimpleResponse, reqwest::Error> {
        self.post_simple_command(VEHICLE_COMMAND_CHARGE_PORT_DOOR_CLOSE)
    }

    fn post_simple_command(&self, command: &str) -> Result<SimpleResponse, reqwest::Error> {
        let url = self.get_command_url(command);
        let resp: Response<SimpleResponse> = self.tesla_client.client.post(url)
            .send()?
            .json()?;
        Ok(resp.into_response())
    }

    pub fn get(&self) -> Result<Vehicle, reqwest::Error> {
        let resp: Response<Vehicle> = self.tesla_client.client.get(self.get_base_url())
            .send()?
            .json()?;

        Ok(resp.into_response())
    }

    pub fn get_all_data(&self) -> Result<FullVehicleData, reqwest::Error> {
        let url = endpoint_url!(self, VEHICLE_DATA);

        let resp: Response<FullVehicleData> = self.tesla_client.client.get(url)
            .send()?
            .json()?;

        Ok(resp.into_response())
    }

    pub fn get_soc(&self) -> Result<StateOfCharge, reqwest::Error> {
        let url = endpoint_url!(self, VEHICLE_CHARGE_STATE);

        let resp: Response<StateOfCharge> = self.tesla_client.client.get(url)
            .send()?
            .json()?;

        Ok(resp.into_response())
    }

    pub fn get_gui_settings(&self) -> Result<GuiSettings, reqwest::Error> {
        let url = endpoint_url!(self, VEHICLE_GUI_SETTINGS);

        let resp: Response<GuiSettings> = self.tesla_client.client.get(url)
            .send()?
            .json()?;

        Ok(resp.into_response())
    }

    fn get_base_url(&self) -> reqwest::Url {
        let vehicle_path = format!("vehicles/{}/", self.vehicle_id);

        self.tesla_client.api_root
            .join(vehicle_path.as_str())
            .expect("invalid vehicle path")
    }

    fn get_command_url(&self, command: &str) -> reqwest::Url {
        let command_path = format!("vehicles/{}/command/{}", self.vehicle_id, command);

        self.tesla_client.api_root
            .join(command_path.as_str())
            .expect("invalid vehicle path")
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
