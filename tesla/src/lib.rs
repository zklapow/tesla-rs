use reqwest::Client;
use reqwest::header;

pub use models::*;

mod models;

const DEFAULT_BASE_URI: &str = "https://owner-api.teslamotors.com/api/1/";
const ENDPOINT_GET_VEHICLES: &str = "vehicles";
const ENDPOINT_GET_VEHICLE: &str = "vehicles/{}";

pub struct TeslaClient<'a> {
    api_root: &'a str,
    client: Client,
}

pub struct VehicleClient<'a> {
    tesla_client: TeslaClient<'a>,
    vehicle_id: u64,
}

impl<'a> TeslaClient<'a> {
    pub fn default(access_token: &'a str) -> TeslaClient<'a> {
        TeslaClient::new(DEFAULT_BASE_URI, access_token)
    }

    pub fn new(api_root: &'a str, access_token: &'a str) -> TeslaClient<'a> {
        let mut headers = header::HeaderMap::new();

        let auth_value = header::HeaderValue::from_str(format!("Bearer {}", access_token).as_str()).expect("bearer header");

        headers.insert(header::AUTHORIZATION, auth_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Could not create client");

        TeslaClient {
            api_root,
            client,
        }
    }

    pub fn use_vehicle(self, vehicle_id: u64) -> VehicleClient<'a> {
        VehicleClient {
            tesla_client: self,
            vehicle_id
        }
    }

    pub fn get_vehicles(&self) -> Result<Vec<Vehicle>, reqwest::Error> {
        let url = reqwest::Url::parse(DEFAULT_BASE_URI)
            .expect("Cannot parse base url")
            .join(ENDPOINT_GET_VEHICLES)
            .expect("Cannot parse endpoint");

        let vehicle_response: ResponseArray<Vehicle> = self.client.get(url)
            .send()?
            .json()?;

        Ok(vehicle_response.into_response())
    }

    pub fn get(&self, endpoint: String) {
        let full_url: String = self.api_root.to_owned() + endpoint.as_str();
        let res = self.client.get(&full_url)
            .send()
            .expect("req");

        println!("Got result: {:?}", res);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
