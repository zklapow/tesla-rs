use reqwest::Client;
use reqwest::header;

pub use reqwest;

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

// We expect here because this is parsing a const and will not fail
macro_rules! endpoint_url {
    ($client: ident, $e:expr) => {
        $client.get_base_url().join($e).expect("cannot parse endpoint")
    }
}

#[derive(Clone)]
pub struct TeslaClient {
    pub api_root: url::Url,
    client: Client,
}

#[derive(Clone)]
pub struct VehicleClient {
    tesla_client: TeslaClient,
    vehicle_id: u64,
}

impl TeslaClient {
    pub fn default(access_token: &str) -> TeslaClient {
        TeslaClient::new(DEFAULT_BASE_URI, access_token)
    }

    pub fn new(api_root: &str, access_token: &str) -> TeslaClient {
        let mut headers = header::HeaderMap::new();

        let auth_value = header::HeaderValue::from_str(format!("Bearer {}", access_token).as_str()).expect("bearer header");

        headers.insert(header::AUTHORIZATION, auth_value);

        let client = reqwest::Client::builder()
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
            vehicle_id
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

    fn get_base_url(&self) -> url::Url {
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

    fn get_base_url(&self) -> url::Url {
        let vehicle_path = format!("vehicles/{}/", self.vehicle_id);

        self.tesla_client.api_root
            .join(vehicle_path.as_str())
            .expect("invalide vehicle path")
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
