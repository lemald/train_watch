use std::collections::HashSet;

use jsonapi::api::*;
use jsonapi::jsonapi_model;
use jsonapi::model::*;
use serde_derive::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::CarNumberToVehicleId;
use crate::Status;
use crate::VehicleIdToVehicleStatus;
use crate::VehicleStatus;

#[derive(Serialize, Deserialize, Debug)]
struct Carriage {
    label: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Stop {
    id: String,
    name: String,
}

jsonapi_model!(Stop; "stop");

#[derive(Serialize, Deserialize, Debug)]
struct Vehicle {
    id: String,
    label: String,
    current_status: String,
    carriages: Vec<Carriage>,
    stop: Stop,
}

jsonapi_model!(Vehicle; "vehicle"; has one stop);

pub async fn loop_poll_data(
    car_number_to_vehicle_id: &CarNumberToVehicleId,
    vehicle_id_to_vehicle_status: &VehicleIdToVehicleStatus,
) {
    loop {
        let _ = poll_data(&car_number_to_vehicle_id, &vehicle_id_to_vehicle_status).await;

        sleep(Duration::from_secs(5)).await;
    }
}

async fn poll_data(
    car_number_to_vehicle_id: &CarNumberToVehicleId,
    vehicle_id_to_vehicle_status: &VehicleIdToVehicleStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://api-v3.mbta.com/vehicles?filter[route]=Red&include=stop")
        .await?
        .json::<DocumentData>()
        .await?;

    if let Some(PrimaryData::Multiple(vehicles)) = resp.data {
        let mut car_number_to_vehicle_id = car_number_to_vehicle_id.lock().unwrap();
        let mut vehicle_id_to_vehicle_status = vehicle_id_to_vehicle_status.lock().unwrap();

        let mut car_numbers_present = HashSet::new();
        let mut vehicle_ids_present = HashSet::new();

        for vehicle in vehicles.iter() {
            let vehicle = Vehicle::from_jsonapi_resource(&vehicle, &resp.included)?;

            vehicle_ids_present.insert(vehicle.id.clone());

            vehicle_id_to_vehicle_status.insert(
                vehicle.id.clone(),
                VehicleStatus {
                    status: current_status_to_status(&vehicle.current_status)?,
                    station_name: vehicle.stop.name.clone(),
                },
            );

            for carriage in vehicle.carriages.iter() {
                car_number_to_vehicle_id.insert(carriage.label.clone(), vehicle.id.clone());
                car_numbers_present.insert(carriage.label.clone());
            }
        }

        vehicle_id_to_vehicle_status.retain(|k, _| vehicle_ids_present.contains(k));
        car_number_to_vehicle_id.retain(|k, _| car_numbers_present.contains(k));

        println!("{:#?}", vehicle_id_to_vehicle_status);
    }

    Ok(())
}

fn current_status_to_status(current_status: &str) -> Result<Status, &str> {
    match current_status {
        "STOPPED_AT" => Ok(Status::StoppedAt),
        "IN_TRANSIT_TO" => Ok(Status::InTransitTo),
        "INCOMING_AT" => Ok(Status::IncomingAt),
        _ => Err("Invalid status"),
    }
}
