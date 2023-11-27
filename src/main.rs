mod mbta_api;
mod routes;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mbta_api::loop_poll_data;
use routes::routes;

type CarNumber = String;
type VehicleId = String;
type CarNumberToVehicleId = Arc<Mutex<HashMap<CarNumber, VehicleId>>>;

#[tokio::main]
async fn main() {
    let car_number_to_vehicle_id: CarNumberToVehicleId = Arc::new(Mutex::new(HashMap::new()));

    {
        let car_number_to_vehicle_id = car_number_to_vehicle_id.clone();

        tokio::spawn(async move {
            loop_poll_data(&car_number_to_vehicle_id).await;
        });
    }

    warp::serve(routes()).run(([127, 0, 0, 1], 3030)).await;
}
