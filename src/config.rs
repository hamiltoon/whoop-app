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
