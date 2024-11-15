use serde::{Deserialize, Serialize};


#[derive(sqlx::Type, Clone, Copy, Deserialize, Serialize, Debug)]
#[sqlx(type_name = "language")]
pub enum Language {
    VN,
    CH,
}
