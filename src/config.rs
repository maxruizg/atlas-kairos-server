use std::env;

/// Application-wide configuration resolved from environment variables.
///
/// All fields have safe defaults so the server can start without a .env
/// file (useful in CI/CD environments where vars are injected directly).
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub frontend_url: String,
}

impl Config {
    /// Build a [`Config`] by reading environment variables.
    ///
    /// Missing variables fall back to the documented defaults rather than
    /// panicking, so operators only need to override what differs from
    /// the defaults.
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8080);

        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        Self {
            host,
            port,
            frontend_url,
        }
    }

    /// Returns the full bind address string expected by Actix-web.
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
