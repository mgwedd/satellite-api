use std::env;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub express_cores: usize,
    pub heavy_cores: usize,
    pub redis_url: Option<String>,
}

pub type Config = ServerConfig;

impl ServerConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            express_cores: env::var("MAX_EXPRESS_CORES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            heavy_cores: env::var("MAX_HEAVY_CORES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            redis_url: env::var("REDIS_URL").ok(),
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid HOST or PORT configuration")
    }
}
