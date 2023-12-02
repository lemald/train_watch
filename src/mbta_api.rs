use std::collections::HashSet;

use jsonapi::api::*;
use jsonapi::jsonapi_model;
use jsonapi::model::*;
use serde_derive::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::CarNumberToVehicleId;

#[derive(Serialize, Deserialize, Debug)]
struct Carriage {
    label: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vehicle {
    id: String,
    label: String,
    carriages: Vec<Carriage>,
}

jsonapi_model!(Vehicle; "vehicle");

pub async fn loop_poll_data(car_number_to_vehicle_id: &CarNumberToVehicleId) {
    loop {
        let _ = poll_data(&car_number_to_vehicle_id).await;

        {
            let mut car_number_to_vehicle_id = car_number_to_vehicle_id.lock().unwrap();

            car_number_to_vehicle_id.insert("1848".to_string(), "123456".to_string());
        }

        sleep(Duration::from_secs(5)).await;
    }
}

async fn poll_data(
    car_number_to_vehicle_id: &CarNumberToVehicleId,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://api-v3.mbta.com/vehicles?filter[route]=Red")
        .await?
        .json::<DocumentData>()
        .await?;

    if let Some(data) = resp.data {
        if let PrimaryData::Multiple(vehicles) = data {
            let mut car_number_to_vehicle_id = car_number_to_vehicle_id.lock().unwrap();
            let mut car_numbers_present = HashSet::new();

            for vehicle in vehicles.iter() {
                let vehicle = Vehicle::from_jsonapi_resource(&vehicle, &None)?;

                for carriage in vehicle.carriages.iter() {
                    car_number_to_vehicle_id.insert(carriage.label.clone(), vehicle.id.clone());
                    car_numbers_present.insert(carriage.label.clone());
                }
            }

            car_number_to_vehicle_id.retain(|k, _| car_numbers_present.contains(&k.to_string()));

            println!("{:#?}", car_number_to_vehicle_id);
        }
    }

    Ok(())
}
