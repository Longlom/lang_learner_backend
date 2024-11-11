use std::env;

use auth::{json_register_body, register};
use warp::Filter;

pub mod auth;

#[tokio::main]
async fn main() {

    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "lang_learner_backend::auth=debug");
    }

    pretty_env_logger::init();
    let api_path = warp::path("api");

    let register_path = api_path
        .and(warp::path("register"))
        .and(warp::post())
        .and(json_register_body())
        .and_then(register);

    let routes = register_path.with(warp::log("lang_learner_backend::auth"));



    warp::serve(routes).run(([127,0,0,1], 3000)).await;
}
