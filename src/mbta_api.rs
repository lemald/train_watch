use tokio::time::{sleep, Duration};

use crate::CarNumberToVehicleId;

pub async fn loop_poll_data(car_number_to_vehicle_id: &CarNumberToVehicleId) {
    {
        let mut car_number_to_vehicle_id = car_number_to_vehicle_id.lock().unwrap();

        car_number_to_vehicle_id.insert("1848".to_string(), "123456".to_string());
    }

    sleep(Duration::from_secs(5)).await;
}
