use warp::Filter;

#[tokio::main]
async fn main() {
    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./static/index.html"));

    let static_content = warp::get()
        .and(warp::path("static"))
        .and(warp::fs::dir("./static"));

    let routes = index.or(static_content);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
