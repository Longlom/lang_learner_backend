use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use warp::Filter;

#[derive(Clone, Debug)]
pub struct Database {
    pub conn_pool: Pool<Postgres>
}

impl  Database {
    pub async fn init() -> Self {
        let db_name = match std::env::var("POSTGRES_DB") {
            Ok(url) => url,
            Err(err) => {
                log::error!("Cannot find POSTGRES_DB variable {}", err);
                panic!();
            }
        };

        let user =match std::env::var("POSTGRES_USER") {
            Ok(url) => url,
            Err(err) => {
                log::error!("Cannot find POSTGRES_USER variable {}", err);
                panic!();
            }
        };

        let pass =match std::env::var("POSTGRES_PASSWORD") {
            Ok(url) => url,
            Err(err) => {
                log::error!("Cannot find POSTGRES_PASSWORD variable {}", err);
                panic!();
            }
        };


        let db_url = format!("postgres://{user}:{pass}@localhost:5432/{db_name}");


        let pool = match PgPoolOptions::new().connect(&db_url).await {
            Ok(conn) => conn,
            Err(err) => {
                log::error!("Cannot connect to database  {}", err);
                panic!();
            }
        };

        log::info!("DB initialized");

        Self {
            conn_pool: pool,
        }
    }


    pub fn with_db(db: Self) -> impl Filter<Extract = (Self,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}