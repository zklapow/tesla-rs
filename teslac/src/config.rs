use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub influx: Option<InfluxConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub api_token: String,
    pub default_vehicle: Option<String>,
    pub logspec: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InfluxConfig {
    pub url: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub interval: Option<u64>
}