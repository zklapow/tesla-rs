use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use influx_db_client::{InfluxClient, Point, Points, Precision, Value};
use influx_db_client::Error as InfluxError;

use tesla::{TeslaClient, Vehicle};

use crate::config::InfluxConfig;

pub fn run_influx_reporter(cfg: InfluxConfig, vehicle_name: String, client: TeslaClient) {
    let vehicle = client.get_vehicle_by_name(vehicle_name.as_str())
        .ok()
        .expect("could not find vehicle")
        .expect("could not find vehicle");

    let vclient = client.vehicle(vehicle.id);
    let influxc = get_influx_client(cfg);

    let running = Arc::new(AtomicBool::new(true));

    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting ctrl-c handler");

    while running.load(Ordering::SeqCst) {
        match vclient.get() {
            Ok(state) => {
                match report_state(state, &influxc) {
                    Ok(_) => debug!("Successfully sent measurement to influx"),
                    Err(e) => error!("Error reporting to influx: {:?}", e)
                }
            }
            Err(e) => warn!("Error fetching vehicle state: {:?}", e)
        }

        info!("Running");
        sleep(Duration::from_secs(1));
    }
}

fn report_state(state: Vehicle, client: &InfluxClient) -> Result<(), InfluxError> {
    let mut meas: Point = point!("state");
    meas.add_field("value", Value::String(state.state));

    let id_s = state.id.to_string();
    let vid_s = state.vehicle_id.to_string();
    meas.add_tag("id", Value::Integer(state.id as i64));
    meas.add_tag("vehicle_id", Value::Integer(state.vehicle_id as i64));
    meas.add_tag("display_name", Value::String(state.display_name));
    meas.add_tag("vin", Value::String(state.vin));

    client.write_point(meas, Some(Precision::Milliseconds), None)
}

fn get_influx_client(cfg: InfluxConfig) -> InfluxClient {
    let mut client = InfluxClient::new(cfg.url.unwrap_or("http://localhost:8086".to_string()), cfg.database.unwrap_or("default".to_string()));

    if cfg.user.is_some() {
        return client.set_authentication(cfg.user.unwrap(), cfg.password.unwrap_or("".to_owned()));
    }

//    let creds = Credentials {
//        username: cfg.user.as_ref().map_or("tesla", |x| &**x),
//        password: cfg.password.as_ref().map_or("", |x| &**x),
//        database: cfg.database.as_ref().map_or("tesla", |x| &**x),
//    };
//
//    let host = cfg.url.as_ref().map_or("http://localhost:8086", |x| &**x);
//    let hosts = vec![host];
//    let client = create_client(creds, hosts);

    client
}

