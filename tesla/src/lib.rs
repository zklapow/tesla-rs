use std::collections::HashMap;

use http::StatusCode;
pub use reqwest;
use reqwest::blocking::Client;
use reqwest::header;
use serde::de::DeserializeOwned;

pub use models::*;
pub use tesla_rs_error::*;

mod tesla_rs_error;
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
    pub fn authenticate(email: &str, password: &str) -> Result<String, TeslaError> {
        TeslaClient::authenticate_using_api_root(DEFAULT_BASE_URI, email, password)
    }

    pub fn authenticate_using_api_root(api_root: &str, email: &str, password: &str) -> Result<String, TeslaError> {
        let map = TeslaClient::get_auth_params(email, password);
        let resp = TeslaClient::call_auth_route(api_root, map)?;

        let expires_in_days = resp.expires_in / 60 / 60 / 24;
        println!("The access token will expire in {} days", expires_in_days);
        Ok(resp.access_token)
    }

    fn get_auth_params<'a>(email: &'a str, password: &'a str) -> HashMap<&'a str, &'a str> {
        let mut map = HashMap::new();
        map.insert("grant_type", "password");
        // Use client_id and client_secret obtained from Android/iOS app. These are not the user's api key.
        map.insert("client_id", "81527cff06843c8634fdc09e8ac0abefb46ac849f38fe1e431c2ef2106796384");
        map.insert("client_secret", "c7257eb71a564034f9419ee651c7d0e5f7aa6bfbd18bafb5c5c033b093bb2fa3");
        map.insert("email", email);
        map.insert("password", password);
        map
    }

    fn call_auth_route(api_root: &str, params_map: HashMap<&str, &str>) -> Result<AuthResponse, TeslaError> {
        let root_url = reqwest::Url::parse(api_root).expect("Could not parse API root");
        let url = root_url.join("/oauth/token").expect("Could not parse API endpoint");
        let client = Client::new();
        let response = client.post(url).json(&params_map).send()?;
        match response.status() {
            StatusCode::OK => Ok(response.json()?),
            StatusCode::UNAUTHORIZED => Err(TeslaError::AuthError), // TODO : Possibly copy response.text() into the error object.
            _ => {
                Err(TeslaError::from(AppError {
                    message: format!("Error while authenticating : {}", response.text()?)
                }))
            }
        }
    }

    pub fn default(access_token: &str) -> TeslaClient {
        TeslaClient::new(DEFAULT_BASE_URI, access_token)
    }

    pub fn new(api_root: &str, access_token: &str) -> TeslaClient {
        let mut headers = header::HeaderMap::new();

        let auth_value = header::HeaderValue::from_str(format!("Bearer {}", access_token).as_str()).unwrap();

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

    pub fn get_vehicles(&self) -> Result<Vec<Vehicle>, TeslaError> {
        let url = endpoint_url!(self, ENDPOINT_GET_VEHICLES);
        let response = self.client.get(url).send()?;
        if response.status() == 200 {
            let vehicle_response: ResponseArray<Vehicle> = response.json()?;
            Ok(vehicle_response.into_response())
        } else {
            Err(self.get_error_from_response(response))
        }
    }

    pub fn get_vehicle_by_name(&self, name: &str) -> Result<Option<Vehicle>, TeslaError> {
        let vehicle = self.get_vehicles()?.into_iter()
            .find(|v| v.display_name.to_lowercase() == name.to_lowercase());

        Ok(vehicle)
    }

    fn get_base_url(&self) -> reqwest::Url {
        self.api_root.clone()
    }

    fn get_error_from_response(&self, response: reqwest::blocking::Response) -> TeslaError {
        let headers = response.headers();
        let mut err = TeslaError::ParseAppError(AppError {
            message: "Unspecified error".to_owned()
        });
        if response.status() == 401 {
            let header_value = headers.get("www-authenticate");
            if header_value.is_some() {
                if header_value.unwrap().to_str().unwrap_or("").contains("invalid_token") {
                    err = TeslaError::InvalidTokenError;
                }
            }
        } else if response.status() == 404 {
            err = TeslaError::ParseAppError(AppError {
                message: "Not found error (404)".to_owned()
            });
        }
        err
    }
}

impl VehicleClient {
    pub fn wake_up(&self) -> Result<Vehicle, TeslaError> {
        let url = endpoint_url!(self, VEHICLE_COMMAND_WAKE);

        let response = self.tesla_client.client.post(url).send()?;
        if response.status() == 200 {
            let resp: Response<Vehicle> = response.json()?;
            Ok(resp.into_response())
        } else {
            Err(self.tesla_client.get_error_from_response(response))
        }
    }

    pub fn flash_lights(&self) -> Result<SimpleResponse, TeslaError> {
        self.post_simple_command(VEHICLE_COMMAND_FLASH)
    }

    pub fn door_unlock(&self) -> Result<SimpleResponse, TeslaError> {
        self.post_simple_command(VEHICLE_COMMAND_DOOR_UNLOCK)
    }

    pub fn door_lock(&self) -> Result<SimpleResponse, TeslaError> {
        self.post_simple_command(VEHICLE_COMMAND_DOOR_LOCK)
    }

    pub fn honk_horn(&self) -> Result<SimpleResponse, TeslaError> {
        self.post_simple_command(VEHICLE_COMMAND_HONK_HORN)
    }

    pub fn auto_conditioning_start(&self) -> Result<SimpleResponse, TeslaError> {
        self.post_simple_command(VEHICLE_COMMAND_AUTO_CONDITIONING_START)
    }

    pub fn auto_conditioning_stop(&self) -> Result<SimpleResponse, TeslaError> {
        self.post_simple_command(VEHICLE_COMMAND_AUTO_CONDITIONING_STOP)
    }

    pub fn remote_start_drive(&self) -> Result<SimpleResponse, TeslaError> {
        // TODO : Need to pass the password in the querystring
        let url = self.get_command_url(VEHICLE_COMMAND_REMOTE_START_DRIVE);
        let response = self.tesla_client.client.post(url).send()?;
        if response.status() == 200 {
            let resp: Response<SimpleResponse> = response.json()?;
            Ok(resp.into_response())
        } else {
            Err(self.tesla_client.get_error_from_response(response))
        }
    }

    pub fn charge_port_door_open(&self) -> Result<SimpleResponse, TeslaError> {
        self.post_simple_command(VEHICLE_COMMAND_CHARGE_PORT_DOOR_OPEN)
    }

    pub fn charge_port_door_close(&self) -> Result<SimpleResponse, TeslaError> {
        self.post_simple_command(VEHICLE_COMMAND_CHARGE_PORT_DOOR_CLOSE)
    }

    fn post_simple_command(&self, command: &str) -> Result<SimpleResponse, TeslaError> {
        let url = self.get_command_url(command);
        let response = self.tesla_client.client.post(url).send()?;
        if response.status() == 200 {
            let resp: Response<SimpleResponse> = response.json()?;
            Ok(resp.into_response())
        } else {
            Err(self.tesla_client.get_error_from_response(response))
        }
    }

    pub fn get(&self) -> Result<Vehicle, TeslaError> {
        let url = self.get_base_url();
        self.get_some_data(url)
    }

    pub fn get_all_data(&self) -> Result<FullVehicleData, TeslaError> {
        let url = endpoint_url!(self, VEHICLE_DATA);
        self.get_some_data(url)
    }

    pub fn get_soc(&self) -> Result<StateOfCharge, TeslaError> {
        let url = endpoint_url!(self, VEHICLE_CHARGE_STATE);
        self.get_some_data(url)
    }

    pub fn get_gui_settings(&self) -> Result<GuiSettings, TeslaError> {
        let url = endpoint_url!(self, VEHICLE_GUI_SETTINGS);
        self.get_some_data(url)
    }

    fn get_some_data<T: DeserializeOwned>(&self, url: reqwest::Url) -> Result<T, TeslaError> {
        let response = self.tesla_client.client.get(url).send()?;
        if response.status() == 200 {
            let resp: Response<T> = response.json()?;
            Ok(resp.into_response())
        } else {
            Err(self.tesla_client.get_error_from_response(response))
        }
    }

    fn get_base_url(&self) -> reqwest::Url {
        let vehicle_path = format!("vehicles/{}/", self.vehicle_id);

        self.tesla_client.api_root
            .join(vehicle_path.as_str())
            .unwrap()
    }

    fn get_command_url(&self, command: &str) -> reqwest::Url {
        let command_path = format!("vehicles/{}/command/{}", self.vehicle_id, command);

        self.tesla_client.api_root
            .join(command_path.as_str())
            .unwrap()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
