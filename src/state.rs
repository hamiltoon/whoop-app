use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::config::Config;
use crate::whoop::models::TokenPair;

pub type SessionId = Uuid;
pub type TokenStore = Arc<RwLock<HashMap<SessionId, TokenPair>>>;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub tokens: TokenStore,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tokens: Arc::new(RwLock::new(HashMap::new())),
            http_client: reqwest::Client::new(),
        }
    }
}
