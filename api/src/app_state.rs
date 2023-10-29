use sea_orm::DatabaseConnection;

pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_sec: String,
    pub jwt_iss: String,
    pub jwt_aud: String
}