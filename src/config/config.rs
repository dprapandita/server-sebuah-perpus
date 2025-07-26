use std::env;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
}

impl Config {
    pub fn init() -> Config {
        dotenvy::dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let host = env::var("HOST").expect("HOST is not set in .env file");
        let port = env::var("PORT").expect("PORT is not set in .env file");
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is not set in .env file");
        Config {
            database_url,
            host,
            port: port.parse::<u16>().unwrap(),
            jwt_secret,
        }
    }
}