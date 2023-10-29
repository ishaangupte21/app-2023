// Routes under the /colleges path

use actix_web::{get, web, HttpResponse};

use crate::app_state::AppState;

#[get("/list-all")]
pub async fn hande_list_all_colleges(state: web::Data<AppState>) -> HttpResponse {
    
}