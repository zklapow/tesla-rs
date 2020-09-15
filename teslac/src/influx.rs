use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use influx_db_client::{InfluxClient, Point, Precision, Value};
use snafu::ResultExt;

use tesla::{TeslaClient, Vehicle, VehicleClient, StateOfCharge, VehicleState, ClimateState, DriveState};

use crate::config::InfluxConfig;
use crate::error::{Error, TeslaApi, InfluxWrite};

pub fn run_influx_reporter(cfg: InfluxConfig, vehicle_name: String, client: TeslaClient) -> Result<(), Error> {
    let vehicle = client.get_vehicle_by_name(vehicle_name.as_str())
        .ok()
        .expect("could not find vehicle")
        .expect("could not find vehicle");

    let vclient = client.vehicle(vehicle.id);
    let influxc = get_influx_client(cfg.clone());

    let running = Arc::new(AtomicBool::new(true));

    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting ctrl-c handler");

    let poll_duration = cfg.interval.unwrap_or(30);

    let mut  next_poll_time = Instant::now();
    while running.load(Ordering::SeqCst) {
        if Instant::now() > next_poll_time {
            debug!("Reporting to influx");
            check_and_report(&vclient, &influxc)?;

            next_poll_time = Instant::now() + Duration::from_secs(poll_duration);
        }

        sleep(Duration::from_millis(poll_duration));
    }

    Ok(())
}

fn check_and_report(client: &VehicleClient, influx: &InfluxClient) -> Result<(), Error> {
    info!("Attempting to fetch car data and report to influx");
    let state = client.get().context(TeslaApi)?;
    report_state(&state, &influx)?;

    match state.state.as_str() {
        "online" => {
            report_online(client, &state, influx)
        },
        "offline" | "asleep" => {
            Ok(())
        },
        _ => {
            Err(Error::UnknownState { state: state.state })
        }
    }
}

fn report_online(client: &VehicleClient, vehicle: &Vehicle, influx: &InfluxClient) -> Result<(), Error> {
    info!("Vehicle is online, reporting full data to influx");
    let all_data = client.get_all_data().context(TeslaApi)?;
    debug!("Fetched all vehicle data: {:?}", all_data);

    report_soc(vehicle, &all_data.charge_state, influx)?;
    report_odo(vehicle, &all_data.vehicle_state, influx)?;
    report_temp(vehicle, &all_data.climate_state, influx)?;
    report_loc(vehicle, &all_data.drive_state, influx)?;

    Ok(())
}

fn report_loc(vehicle: &Vehicle, drive_state: &DriveState, client: &InfluxClient) -> Result<(), Error> {
    let mut loc: Point = point!("location");

    loc.add_field("latitude", Value::Float(drive_state.latitude));
    loc.add_field("longitude", Value::Float(drive_state.longitude));
    loc.add_field("heading", Value::Integer(drive_state.heading as i64));

    write_point(loc, vehicle, client)
}

fn report_temp(vehicle: &Vehicle, climate_state: &ClimateState, client: &InfluxClient) -> Result<(), Error> {
    let mut temp: Point = point!("temperature");

    temp.add_field("inside", Value::Float(climate_state.inside_temp));
    temp.add_field("outside", Value::Float(climate_state.outside_temp));

    write_point(temp, vehicle, client)
}

fn report_odo(vehicle: &Vehicle, vehicle_state: &VehicleState, client: &InfluxClient) -> Result<(), Error> {
    let mut odo: Point = point!("odometer");

    odo.add_field("value", Value::Float(vehicle_state.odometer));

    write_point(odo, vehicle, client)
}

fn report_soc(vehicle: &Vehicle, charge_state: &StateOfCharge, client: &InfluxClient) -> Result<(), Error> {
    let mut battery: Point = point!("battery");

    battery.add_field("level", Value::Integer(charge_state.battery_level as i64));
    battery.add_field("range", Value::Float(charge_state.battery_range));
    battery.add_field("range-ideal", Value::Float(charge_state.ideal_battery_range));
    battery.add_field("range-est", Value::Float(charge_state.est_battery_range));

    write_point(battery, vehicle, client)
}

fn report_state(state: &Vehicle, client: &InfluxClient) -> Result<(), Error> {
    let mut meas: Point = point!("state");
    let state_bool = state.state.as_str().to_lowercase() == "online";
    meas.add_field("value", Value::String(state.state.clone()));
    meas.add_field("online", Value::Boolean(state_bool));

    write_point(meas, state, client)
}

fn write_point(mut point: Point, vehicle: &Vehicle, client: &InfluxClient) -> Result<(), Error> {
    add_vehicle_tags(&mut point, &vehicle);

    client.write_point(point, Some(Precision::Milliseconds), None)
        .context(InfluxWrite)
}

fn add_vehicle_tags(point: &mut Point, vehicle: &Vehicle) {
    point.add_tag("id", Value::Integer(vehicle.id as i64));
    point.add_tag("vehicle_id", Value::Integer(vehicle.vehicle_id as i64));
    point.add_tag("display_name", Value::String(vehicle.display_name.clone()));
    point.add_tag("vin", Value::String(vehicle.vin.clone()));
}

fn get_influx_client(cfg: InfluxConfig) -> InfluxClient {
    let client = InfluxClient::new(cfg.url.unwrap_or("http://localhost:8086".to_string()), cfg.database.unwrap_or("default".to_string()));

    if cfg.user.is_some() {
        return client.set_authentication(cfg.user.unwrap(), cfg.password.unwrap_or("".to_owned()));
    }

    client
}

