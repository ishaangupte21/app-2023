use std::{env, io};

use actix_web::{middleware::Logger, web, App, HttpServer};
use app_state::AppState;
use bb8_redis::{bb8, RedisConnectionManager};
use dotenvy::dotenv;
use sea_orm::Database;

mod app_state;
mod jwt;
mod routes;
mod structures;

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().expect("Unable to find .env file");

    let port: u16 = match env::var("PORT") {
        Ok(val) => val.parse::<u16>().expect("Unable to parse PORT as u16"),
        Err(_) => 8000,
    };

    let jwt_secret = env::var("JWT_SECRET").expect("No JWT secret found in .env file");
    let jwt_issuer = env::var("JWT_ISSUER").expect("No JWT_ISSUER in .env file");
    let jwt_audience = env::var("JWT_AUDIENCE").expect("No jwt audience in .env file");

    let database_url = env::var("DATABASE_URL").expect("No DATABASE_URL found in .env file");
    // Here, we must initialize a database connection with seaorm.
    let db = Database::connect(database_url)
        .await
        .expect("Unable to connect to Postgres database");

    let redis_url = env::var("REDIS_URL").expect("No REDIS_URL found in .env file");
    // Initialize the redis connection
    let redis_manager =
        RedisConnectionManager::new(redis_url).expect("Unable to connect to redis instance");
    let redis_pool = bb8::Pool::builder()
        .build(redis_manager)
        .await
        .expect("Unable to initialize redis pool");

    let pos_stack_key = env::var("POS_STACK_KEY").expect("No POS_STACK_KEY in .env file");

    // Now, we can create the universal app state.
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %r %D"))
            .app_data(web::Data::new(AppState {
                db: db.clone(),
                jwt_sec: jwt_secret.clone(),
                jwt_iss: jwt_issuer.clone(),
                jwt_aud: jwt_audience.clone(),
                redis_pool: redis_pool.clone(),
                pos_stack_key: pos_stack_key.clone(),
            }))
            .service(routes::handle_root_path)
            .service(routes::auth::handle_google_login)
            .service(routes::auth::handle_verify_access_token)
            .service(routes::colleges::hande_list_all_colleges)
            .service(routes::colleges::handle_get_colleges_with_params)
            .service(routes::colleges::handle_get_single_college_info)
            .service(routes::colleges::handle_how_reviewed_route)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
