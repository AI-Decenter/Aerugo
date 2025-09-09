use serde::{Deserialize, Serialize};
use secrecy::{Secret, ExposeSecret};
use validator::Validate;
use std::net::SocketAddr;
use url::Url;
use anyhow::{Result, Context};
use std::env;

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct Settings {
    #[validate]
    pub server: ServerSettings,
    #[validate]
    pub database: DatabaseSettings,
    #[validate]
    pub storage: StorageSettings,
    #[validate]
    pub cache: CacheSettings,
    #[validate]
    pub auth: AuthSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ServerSettings {
    pub bind_address: String,
    #[validate(range(min = 1024, max = 65535))]
    pub port: u16,
    pub api_prefix: String,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct DatabaseSettings {
    pub host: String,
    #[validate(range(min = 1024, max = 65535))]
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub database_name: String,
    pub require_ssl: bool,
    pub min_connections: u32,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct StorageSettings {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: Secret<String>,
    pub secret_access_key: Secret<String>,
    pub use_path_style: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CacheSettings {
    pub redis_url: String,
    pub pool_size: u32,
    pub ttl_seconds: u64,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct AuthSettings {
    pub jwt_secret: Secret<String>,
    #[validate(range(min = 300))] // Minimum 5 minutes
    pub jwt_expiration_seconds: u64,
    pub refresh_token_expiration_seconds: u64,
}

impl Settings {
    /// Load configuration completely from environment variables
    /// No default configuration files are used
    pub fn load() -> Result<Self> {
        // Load .env file if it exists (for development)
        dotenv::dotenv().ok();

        // Check for required environment variables and provide helpful error messages
        Self::check_required_env_vars()?;

        let settings = Settings {
            server: ServerSettings::from_env()?,
            database: DatabaseSettings::from_env()?,
            storage: StorageSettings::from_env()?,
            cache: CacheSettings::from_env()?,
            auth: AuthSettings::from_env()?,
        };

        // Validate all settings
        settings.validate_all()
            .context("Configuration validation failed")?;

        Ok(settings)
    }

    /// Check if all required environment variables are set
    fn check_required_env_vars() -> Result<()> {
        let required_vars = vec![
            // Server
            "LISTEN_ADDRESS",
            "LOG_LEVEL",
            
            // Database
            "DATABASE_URL",
            
            // Storage (S3/MinIO)
            "S3_ENDPOINT",
            "S3_BUCKET",
            "S3_ACCESS_KEY",
            "S3_SECRET_KEY",
            "S3_REGION",
            
            // Cache
            "REDIS_URL",
            
            // Auth
            "JWT_SECRET",
        ];

        let mut missing_vars = Vec::new();
        
        for var in required_vars {
            if env::var(var).is_err() {
                missing_vars.push(var);
            }
        }

        if !missing_vars.is_empty() {
            return Err(anyhow::anyhow!(
                "Missing required environment variables: {}. Please check your .env file or environment configuration.",
                missing_vars.join(", ")
            ));
        }

        Ok(())
    }

    pub fn validate_all(&self) -> Result<(), validator::ValidationErrors> {
        self.validate()?;
        self.server.validate()?;
        self.database.validate()?;
        self.storage.validate()?;
        self.cache.validate()?;
        self.auth.validate()?;
        Ok(())
    }
}

impl ServerSettings {
    fn from_env() -> Result<Self> {
        let listen_address = env::var("LISTEN_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        
        // Parse address to extract host and port
        let socket_addr: SocketAddr = listen_address.parse()
            .context("Invalid LISTEN_ADDRESS format. Expected format: 'host:port'")?;
        
        Ok(ServerSettings {
            bind_address: socket_addr.ip().to_string(),
            port: socket_addr.port(),
            api_prefix: env::var("API_PREFIX")
                .unwrap_or_else(|_| "/api/v1".to_string()),
        })
    }
}

impl DatabaseSettings {
    fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable is required")?;
        
        // Parse the database URL to extract components
        let url = Url::parse(&database_url)
            .context("Invalid DATABASE_URL format")?;
        
        let host = url.host_str()
            .context("No host found in DATABASE_URL")?
            .to_string();
        
        let port = url.port()
            .unwrap_or(5432);
        
        let username = url.username().to_string();
        let password = Secret::new(
            url.password()
                .context("No password found in DATABASE_URL")?
                .to_string()
        );
        
        let database_name = url.path()
            .trim_start_matches('/')
            .to_string();

        Ok(DatabaseSettings {
            host,
            port,
            username,
            password,
            database_name,
            require_ssl: env::var("DATABASE_REQUIRE_SSL")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .map(|v| v.parse().unwrap_or(5))
                .unwrap_or(5),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .map(|v| v.parse().unwrap_or(20))
                .unwrap_or(20),
        })
    }
}

impl StorageSettings {
    fn from_env() -> Result<Self> {
        let endpoint = env::var("S3_ENDPOINT")
            .context("S3_ENDPOINT environment variable is required")?;
        
        // Basic URL validation
        Url::parse(&endpoint)
            .context("S3_ENDPOINT must be a valid URL")?;
            
        Ok(StorageSettings {
            endpoint,
            region: env::var("S3_REGION")
                .context("S3_REGION environment variable is required")?,
            bucket: env::var("S3_BUCKET")
                .context("S3_BUCKET environment variable is required")?,
            access_key_id: Secret::new(
                env::var("S3_ACCESS_KEY")
                    .context("S3_ACCESS_KEY environment variable is required")?
            ),
            secret_access_key: Secret::new(
                env::var("S3_SECRET_KEY")
                    .context("S3_SECRET_KEY environment variable is required")?
            ),
            use_path_style: env::var("S3_USE_PATH_STYLE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
        })
    }
}

impl CacheSettings {
    fn from_env() -> Result<Self> {
        Ok(CacheSettings {
            redis_url: env::var("REDIS_URL")
                .context("REDIS_URL environment variable is required")?,
            pool_size: env::var("REDIS_POOL_SIZE")
                .map(|v| v.parse().unwrap_or(10))
                .unwrap_or(10),
            ttl_seconds: env::var("REDIS_TTL_SECONDS")
                .map(|v| v.parse().unwrap_or(3600))
                .unwrap_or(3600),
        })
    }
}

impl AuthSettings {
    fn from_env() -> Result<Self> {
        Ok(AuthSettings {
            jwt_secret: Secret::new(
                env::var("JWT_SECRET")
                    .context("JWT_SECRET environment variable is required")?
            ),
            jwt_expiration_seconds: env::var("JWT_EXPIRATION_SECONDS")
                .map(|v| v.parse().unwrap_or(3600))
                .unwrap_or(3600),
            refresh_token_expiration_seconds: env::var("REFRESH_TOKEN_EXPIRATION_SECONDS")
                .map(|v| v.parse().unwrap_or(604800))
                .unwrap_or(604800), // 7 days
        })
    }
}
impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        let ssl_mode = if self.require_ssl { "require" } else { "prefer" };
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name,
            ssl_mode
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_settings_load_with_env_vars() {
        // Set required environment variables for testing
        env::set_var("LISTEN_ADDRESS", "127.0.0.1:8080");
        env::set_var("LOG_LEVEL", "debug");
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("S3_ENDPOINT", "http://localhost:9000");
        env::set_var("S3_BUCKET", "test-bucket");
        env::set_var("S3_ACCESS_KEY", "test-access");
        env::set_var("S3_SECRET_KEY", "test-secret");
        env::set_var("S3_REGION", "us-east-1");
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var("JWT_SECRET", "test-jwt-secret");

        let settings = Settings::load().expect("Failed to load settings");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.server.api_prefix, "/api/v1");
        assert_eq!(settings.database.host, "localhost");
        assert_eq!(settings.storage.bucket, "test-bucket");
    }

    #[test]
    fn test_missing_required_env_vars() {
        // Clear environment variables
        env::remove_var("LISTEN_ADDRESS");
        env::remove_var("DATABASE_URL");
        
        let result = Settings::load();
        assert!(result.is_err());
    }

    #[test]
    fn test_settings_validation() {
        // Set valid environment variables
        env::set_var("LISTEN_ADDRESS", "127.0.0.1:8080");
        env::set_var("LOG_LEVEL", "debug");
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("S3_ENDPOINT", "http://localhost:9000");
        env::set_var("S3_BUCKET", "test-bucket");
        env::set_var("S3_ACCESS_KEY", "test-access");
        env::set_var("S3_SECRET_KEY", "test-secret");
        env::set_var("S3_REGION", "us-east-1");
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var("JWT_SECRET", "test-jwt-secret");

        let settings = Settings::load().expect("Failed to load settings");
        assert!(settings.validate_all().is_ok());
    }
}
