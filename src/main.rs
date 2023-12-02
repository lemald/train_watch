mod mbta_api;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
        let vehicle_id_to_vehicle_status = vehicle_id_to_vehicle_status.clone();

        tokio::spawn(async move {
            loop_poll_data(&car_number_to_vehicle_id, &vehicle_id_to_vehicle_status).await;
        });
    }

    warp::serve(filters::train_watch(
        car_number_to_vehicle_id,
        vehicle_id_to_vehicle_status,
    ))
    .run(([127, 0, 0, 1], 3030))
    .await;
}

mod filters {
    use std::convert::Infallible;

    use crate::handlers;
    use crate::CarNumberToVehicleId;
    use crate::VehicleIdToVehicleStatus;

    use warp::Filter;

    fn index() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path::end())
            .and(warp::fs::file("./static/index.html"))
    }

    fn static_content() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::get()
            .and(warp::path("static"))
            .and(warp::fs::dir("./static"))
    }

    fn search(
        car_number_to_vehicle_id: CarNumberToVehicleId,
        vehicle_id_to_vehicle_status: VehicleIdToVehicleStatus,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path("search"))
            .and(warp::path::end())
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::form())
            .and(with_car_number_to_vehicle_id(car_number_to_vehicle_id))
            .and(with_vehicle_id_to_vehicle_status(
                vehicle_id_to_vehicle_status,
            ))
            .map(handlers::search)
    }

    fn with_car_number_to_vehicle_id(
        car_number_to_vehicle_id: CarNumberToVehicleId,
    ) -> impl Filter<Extract = (CarNumberToVehicleId,), Error = Infallible> + Clone {
        warp::any().map(move || car_number_to_vehicle_id.clone())
    }

    fn with_vehicle_id_to_vehicle_status(
        vehicle_id_to_vehicle_status: VehicleIdToVehicleStatus,
    ) -> impl Filter<Extract = (VehicleIdToVehicleStatus,), Error = Infallible> + Clone {
        warp::any().map(move || vehicle_id_to_vehicle_status.clone())
    }

    pub fn train_watch(
        car_number_to_vehicle_id: CarNumberToVehicleId,
        vehicle_id_to_vehicle_status: VehicleIdToVehicleStatus,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        index().or(static_content()).or(search(
            car_number_to_vehicle_id.clone(),
            vehicle_id_to_vehicle_status.clone(),
        ))
    }
}

mod handlers {
    use std::collections::HashMap;
    use std::convert::Infallible;

    use crate::CarNumberToVehicleId;
    use crate::VehicleIdToVehicleStatus;

    pub fn search(
        form: HashMap<String, String>,
        car_number_to_vehicle_id: CarNumberToVehicleId,
        vehicle_id_to_vehicle_status: VehicleIdToVehicleStatus,
    ) -> Result<impl warp::Reply, Infallible> {
        let car_number = form.get("car-number").unwrap();
        let car_number_to_vehicle_id = car_number_to_vehicle_id.lock().unwrap();
        let vehicle_id_to_vehicle_status = vehicle_id_to_vehicle_status.lock().unwrap();

        let vehicle_id = car_number_to_vehicle_id.get(car_number).unwrap();

        let vehicle_status = vehicle_id_to_vehicle_status.get(vehicle_id).unwrap();

        Ok(warp::reply::html(vehicle_status.station_name.to_string()))
    }
}
