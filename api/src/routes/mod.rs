// Routes under the root path.
use actix_web::{get, Responder};

pub mod auth;
pub mod colleges;

#[get("/")]
pub async fn handle_root_path() -> impl Responder {
    "App 2023 API v1"
}
