pub mod auth;
pub mod db;

use crate::login::login;
use auth::{
    login::{self, LoginPayload},
    register::{json_register_body, register},
};
use db::db::Database;
use dotenv::dotenv;
use warp::Filter;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let db = Database::init().await;

    let api_path = warp::path("api");

    let register_path = api_path
        .and(warp::path("register"))
        .and(warp::post())
        .and(json_register_body())
        .and(Database::with_db(db.clone()))
        .and_then(register)
        .with(warp::log("lang_learner_backend::auth"));

    let login_path = api_path
        .and(warp::path("login"))
        .and(warp::get())
        .and(warp::query::<LoginPayload>())
        .and(Database::with_db(db.clone()))
        .and_then(login);

    let routes = register_path.or(login_path);

    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
