mod mbta_api;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use warp::Filter;

use mbta_api::loop_poll_data;

type CarNumber = String;
type VehicleId = String;
type CarNumberToVehicleId = Arc<Mutex<HashMap<CarNumber, VehicleId>>>;

#[derive(Debug)]
enum Status {
    StoppedAt,
    InTransitTo,
    IncomingAt,
}

#[derive(Debug)]
struct VehicleStatus {
    status: Status,
    station_name: String,
}

type VehicleIdToVehicleStatus = Arc<Mutex<HashMap<VehicleId, VehicleStatus>>>;

#[tokio::main]
async fn main() {
    let car_number_to_vehicle_id: CarNumberToVehicleId = Arc::new(Mutex::new(HashMap::new()));
    let vehicle_id_to_vehicle_status: VehicleIdToVehicleStatus =
        Arc::new(Mutex::new(HashMap::new()));

    {
        let car_number_to_vehicle_id = car_number_to_vehicle_id.clone();

        tokio::spawn(async move {
            loop_poll_data(&car_number_to_vehicle_id, &vehicle_id_to_vehicle_status).await;
        });
    }

    warp::serve(routes()).run(([127, 0, 0, 1], 3030)).await;
}

fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./static/index.html"));

    let static_content = warp::get()
        .and(warp::path("static"))
        .and(warp::fs::dir("./static"));

    let search = warp::post()
        .and(warp::path("search"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::form())
        .map(
            |form: HashMap<String, String>| match HashMap::get(&form, "car-number") {
                Some(number) => number.clone(),
                None => "No car number given".to_string(),
            },
        )
        .map(warp::reply::html);

    index.or(static_content).or(search)
}
