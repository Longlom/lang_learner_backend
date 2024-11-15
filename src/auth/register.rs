use std::convert::Infallible;

use base64ct::{Base64, Encoding};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use warp::{http::StatusCode, reply::Reply, Filter};

use crate::db::{db::Database, types::Language};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterPayload {
    login: String,
    password: String,
    name: String,
    language: String,
}

pub fn json_register_body(
) -> impl Filter<Extract = (RegisterPayload,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub async fn register(payload: RegisterPayload, db: Database) -> Result<impl Reply, Infallible> {
    log::debug!("register: {:?}", payload);

    let language: Language = match payload.language.as_str() {
        "VN" => {
            Language::VN
        }
        "CH" => {
            Language::CH
        }
        _ => {
            return Ok(StatusCode::BAD_REQUEST);
        }
    };

    let hash = Sha256::digest(&payload.password);
    let password_hash = Base64::encode_string(&hash);

    match sqlx::query(
        "INSERT INTO users (password, login, username, language)
    VALUES ($1, $2, $3, $4::language)",
    )
    .bind(&password_hash)
    .bind(&payload.login)
    .bind(&payload.name)
    .bind(&language)
    .execute(&db.conn_pool)
    .await
    {
        Ok(result) => {
            log::debug!("Successfully created user- {}", result.rows_affected());
        }
        Err(err) => {
            log::error!("Error happened while creating user - {}", err);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    return Ok(StatusCode::CREATED);
}
