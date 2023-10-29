use bb8_redis::{bb8, RedisConnectionManager};
use sea_orm::DatabaseConnection;

pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_sec: String,
    pub jwt_iss: String,
    pub jwt_aud: String,
    pub redis_pool: bb8::Pool<RedisConnectionManager>,
    pub pos_stack_key: String
}
