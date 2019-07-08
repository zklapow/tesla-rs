use reqwest::Client;
use reqwest::header;

pub use models::*;

mod models;

const DEFAULT_BASE_URI: &str = "https://owner-api.teslamotors.com/api/1/";
const ENDPOINT_GET_VEHICLES: &str = "vehicles";
const ENDPOINT_GET_VEHICLE: &str = "vehicles/{}";

const VEHICLE_CHARGE_STATE: &str = "data_request/charge_state";

pub struct TeslaClient {
    pub api_root: url::Url,
    client: Client,
}

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

    pub fn use_vehicle(self, vehicle_id: u64) -> VehicleClient {
        VehicleClient {
            tesla_client: self,
            vehicle_id
        }
    }

    pub fn get_vehicles(&self) -> Result<Vec<Vehicle>, reqwest::Error> {
        let url = self.api_root
            .join(ENDPOINT_GET_VEHICLES)
            .expect("Cannot parse endpoint");

        let vehicle_response: ResponseArray<Vehicle> = self.client.get(url)
            .send()?
            .json()?;

        Ok(vehicle_response.into_response())
    }
}

impl VehicleClient {
    pub fn get_soc(&self) -> Result<StateOfCharge, reqwest::Error> {
        let url = self.get_base_url().join(VEHICLE_CHARGE_STATE).expect("cannot parse endpoint");

        let mut resp: Response<StateOfCharge> = self.tesla_client.client.get(url)
            .send()?
            .json()?;

        println!("Resp: {:?}", resp);

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
