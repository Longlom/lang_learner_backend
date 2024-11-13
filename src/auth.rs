use std::convert::Infallible;

use crate::db::Database;
use base64ct::{Base64, Encoding};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use warp::{http::StatusCode, reply::Reply, Filter};

const ALLOWED_LANGUAGE: [&str; 2] = ["VN", "CH"];

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

    if !ALLOWED_LANGUAGE.contains(&payload.language.as_str()) {
        return Ok(StatusCode::BAD_REQUEST);
    }

    let hash = Sha256::digest(&payload.password);
    let password_hash = Base64::encode_string(&hash);

    match sqlx::query(
        "INSERT INTO users (password, login, username, language)
    VALUES ($1, $2, $3, $4::language)",
    )
    .bind(&password_hash)
    .bind(&payload.login)
    .bind(&payload.name)
    .bind(&payload.language)
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoginPayload {
    login: Option<String>,
    password: Option<String>,
}

#[derive(sqlx::FromRow, Clone, Deserialize, Serialize)]
pub struct UserData {
    username: String,
    language: Language,
}

#[derive(sqlx::Type, Clone, Deserialize, Serialize, Debug)]
#[sqlx(type_name = "language")]
enum Language { VN, CH }

pub async fn login(login_payload: LoginPayload, db: Database) -> Result<impl warp::Reply, Infallible> {
    log::debug!("Login : {:?}", login_payload);

    let mut login;
    let mut password;

    match (login_payload.login, login_payload.password) {
        (Some(l), Some(p)) => {
            login = l;
            password = p;
        }
        _ => {
            log::error!("Missing either password or login");
            return Ok(StatusCode::BAD_REQUEST.into_response());
        }
    }

    let hash = Sha256::digest(&password);
    let password_hash = Base64::encode_string(&hash);
    let user = match sqlx::query_as::<_, UserData>("SELECT username, language FROM users WHERE (password = $1 AND login = $2)")
        .bind(&password_hash)
        .bind(&login)
        .fetch_all(&db.conn_pool).await {
            Ok(res) => {
                if res.len() > 1 {
                    log::error!("Found multiple users with username - {}", login);
                    return Ok(StatusCode::BAD_REQUEST.into_response());
                }

                if res.len() == 0 {
                    return Ok(StatusCode::UNAUTHORIZED.into_response());
                }

            res[0].clone()
            },
            Err(err) => {
                log::error!("Error happened while finding user - {}", err);
                return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };



        let json = warp::reply::json(&user);

        return Ok(warp::reply::with_status(json, StatusCode::OK).into_response());

}
