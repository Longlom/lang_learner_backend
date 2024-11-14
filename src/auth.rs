use std::convert::Infallible;

use crate::db::Database;
use base64ct::{Base64, Encoding};
use chrono::{DateTime, Duration, Local};
use jwt::{token::Signed, Header, SignWithKey, Token};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};
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
    id: i32,
    username: String,
    language: Language,
}

#[derive(sqlx::Type, Clone, Copy, Deserialize, Serialize, Debug)]
#[sqlx(type_name = "language")]
enum Language {
    VN,
    CH,
}
type JwtToken = Token<Header, TokenData, Signed>;

type Time = DateTime<Local>;

#[derive(Deserialize, Serialize)]
pub struct TokenData {
    id: i32,
    language: Language,
    username: String,
    expires_in: Time,
}

enum JwtType {
    ACCESS,
    REFRESH,
}

impl TokenData {
    fn new(user_data: &UserData, token_type: JwtType) -> Self {
        match token_type {
            JwtType::ACCESS => {
                let expires_in = Local::now() + Duration::hours(24);
                return Self {
                    id: user_data.id,
                    language: user_data.language,
                    username: user_data.username.clone(),
                    expires_in,
                };
            }

            JwtType::REFRESH => {
                let expires_in = Local::now() + Duration::hours(168);
                return Self {
                    id: user_data.id,
                    language: user_data.language,
                    username: user_data.username.clone(),
                    expires_in,
                };
            },
        }
    }
}

#[derive(Deserialize, Serialize)]

pub struct TokensData {
    access_token: String,
    refresh_token: String,
}


impl TokensData {
    pub fn new(access_token: JwtToken, refresh_token: JwtToken) -> Self {
        Self {
            access_token: access_token.as_str().to_string(),
            refresh_token: refresh_token.as_str().to_string(),
        }
    }
}

fn create_token(user: UserData) -> Result<TokensData, StatusCode> {
    let key_str = match std::env::var("KEY") {
        Ok(key_str) => key_str,
        Err(_err) => {
            log::error!("KEY must be set");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let key: Hmac<Sha256> = match Hmac::new_from_slice(key_str.as_bytes()) {
        Ok(key) => key,
        Err(err) => {
            log::error!("{}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let header = Header {
        algorithm: jwt::AlgorithmType::Hs256,
        type_: Some(jwt::header::HeaderType::JsonWebToken),
        ..Default::default()
    };

    let access_token = match Token::new(header, TokenData::new(&user, JwtType::ACCESS)).sign_with_key(&key) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("{}", err);
            panic!();
        }
    };

    let key: Hmac<Sha256> = match Hmac::new_from_slice(key_str.as_bytes()) {
        Ok(key) => key,
        Err(err) => {
            log::error!("{}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let header = Header {
        algorithm: jwt::AlgorithmType::Hs256,
        type_: Some(jwt::header::HeaderType::JsonWebToken),
        ..Default::default()
    };

    let refresh_token: JwtToken = match Token::new(header, TokenData::new(&user, JwtType::REFRESH)).sign_with_key(&key) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("{}", err);
            panic!();
        }
    };

    let response = TokensData::new(access_token, refresh_token);

    Ok(response)
}

pub async fn login(
    login_payload: LoginPayload,
    db: Database,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("Login : {:?}", login_payload);

    let login;
    let password;

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
    let user = match sqlx::query_as::<_, UserData>(
        "SELECT id, username, language FROM users WHERE (password = $1 AND login = $2)",
    )
    .bind(&password_hash)
    .bind(&login)
    .fetch_all(&db.conn_pool)
    .await
    {
        Ok(res) => {
            if res.len() > 1 {
                log::error!("Found multiple users with username - {}", login);
                return Ok(StatusCode::BAD_REQUEST.into_response());
            }

            if res.len() == 0 {
                return Ok(StatusCode::UNAUTHORIZED.into_response());
            }

            res[0].clone()
        }
        Err(err) => {
            log::error!("Error happened while finding user - {}", err);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };

    let tokens = create_token(user).unwrap();


    let json = warp::reply::json(&tokens);

    return Ok(warp::reply::with_status(json, StatusCode::OK).into_response());
}
