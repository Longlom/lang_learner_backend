use std::convert::Infallible;

use serde::{Deserialize, Serialize};
use warp::{reply::Reply, Filter};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterPayload {
    login: String,
    password: String,
    name: String,
}

pub fn json_register_body() -> impl Filter<Extract = (RegisterPayload,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub async fn register(payload: RegisterPayload) -> Result<impl Reply, Infallible> {

    log::debug!("register: {:?}", payload);
    Ok("HELLO")

}