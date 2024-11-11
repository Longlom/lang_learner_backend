pub mod auth;
pub mod db;


use std::env;
use dotenv::dotenv;
use auth::{json_register_body, register};
use warp::Filter;


#[tokio::main]
async fn main() {

    dotenv().ok();
    pretty_env_logger::init();

    let db = db::Database::init().await;

    let api_path = warp::path("api");

    let register_path = api_path
        .and(warp::path("register"))
        .and(warp::post())
        .and(json_register_body())
        .and_then(register)
        .with(warp::log("lang_learner_backend::auth"));

    let routes = register_path;

    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
