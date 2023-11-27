use std::collections::HashMap;

use warp::Filter;

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
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
            |simple_map: HashMap<String, String>| match HashMap::get(&simple_map, "car-number") {
                Option::Some(number) => number.clone(),
                Option::None => "No car number given".to_string(),
            },
        );

    index.or(static_content).or(search)
}
