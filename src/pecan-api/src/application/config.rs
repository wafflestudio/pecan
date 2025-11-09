use std::env;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub service: ServiceConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub enable_bg_worker_loop: bool,
    pub max_queue_size: u32,
    pub max_concurrent_executions: u32,
}

pub fn load_config() -> Config {
    let config = Config {
        server: ServerConfig {
            port: get_env_or_default("PORT", 8080),
            host: get_env_or_default("HOST", String::from("0.0.0.0")),
        },
        service: ServiceConfig {
            enable_bg_worker_loop: get_env_or_default("ENABLE_BG_WORKER_LOOP", true),
            max_queue_size: get_env_or_default("MAX_QUEUE_SIZE", 100),
            max_concurrent_executions: get_env_or_default("MAX_CONCURRENT_EXECUTIONS", 20),
        },
    };
    config
}

#[inline]
fn get_env_or_default<T: FromStr>(key: &str, default: T) -> T {
    if let Ok(v) = env::var(key) {
        return v.parse().unwrap_or_else(|_| default);
    }
    default
}
