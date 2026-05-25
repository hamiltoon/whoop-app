use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let client_id = env::var("WHOOP_CLIENT_ID")
            .map_err(|_| "WHOOP_CLIENT_ID is required")?;
        let client_secret = env::var("WHOOP_CLIENT_SECRET")
            .map_err(|_| "WHOOP_CLIENT_SECRET is required")?;
        let redirect_uri = env::var("REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| "PORT must be a valid u16")?;

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
            port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to prevent parallel env-var tests from interfering
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        for key in ["WHOOP_CLIENT_ID", "WHOOP_CLIENT_SECRET", "REDIRECT_URI", "PORT"] {
            env::remove_var(key);
        }
    }

    #[test]
    fn config_from_env_with_all_vars() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("WHOOP_CLIENT_ID", "test_id");
        env::set_var("WHOOP_CLIENT_SECRET", "test_secret");
        env::set_var("REDIRECT_URI", "http://example.com/cb");
        env::set_var("PORT", "8080");

        let config = Config::from_env().unwrap();
        assert_eq!(config.client_id, "test_id");
        assert_eq!(config.client_secret, "test_secret");
        assert_eq!(config.redirect_uri, "http://example.com/cb");
        assert_eq!(config.port, 8080);
        clear_env();
    }

    #[test]
    fn config_defaults_for_optional_vars() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("WHOOP_CLIENT_ID", "id");
        env::set_var("WHOOP_CLIENT_SECRET", "secret");

        let config = Config::from_env().unwrap();
        assert_eq!(config.redirect_uri, "http://localhost:3000/auth/callback");
        assert_eq!(config.port, 3000);
        clear_env();
    }

    #[test]
    fn config_missing_client_id() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("WHOOP_CLIENT_SECRET", "secret");

        let err = Config::from_env().unwrap_err();
        assert!(err.contains("WHOOP_CLIENT_ID"));
        clear_env();
    }

    #[test]
    fn config_missing_client_secret() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("WHOOP_CLIENT_ID", "id");

        let err = Config::from_env().unwrap_err();
        assert!(err.contains("WHOOP_CLIENT_SECRET"));
        clear_env();
    }

    #[test]
    fn config_invalid_port() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("WHOOP_CLIENT_ID", "id");
        env::set_var("WHOOP_CLIENT_SECRET", "secret");
        env::set_var("PORT", "not_a_number");

        let err = Config::from_env().unwrap_err();
        assert!(err.contains("PORT"));
        clear_env();
    }
}
