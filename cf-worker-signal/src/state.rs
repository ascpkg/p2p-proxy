use std::collections::HashMap;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

pub struct AppState {
    pub agents: RwLock<HashMap<String, HashMap<String, Agent>>>,
    pub agent_sdps: RwLock<HashMap<String, Sdp>>,
    pub client_sdps: RwLock<HashMap<String, Sdp>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            client_sdps: RwLock::new(HashMap::new()),
            agent_sdps: RwLock::new(HashMap::new()),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Agent {
    pub uuid: String,
    pub name: String,
    pub os: String,
    #[serde(default)]
    pub last_seen: u64,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Sdp {
    pub sdp: String,
    #[serde(default)]
    pub created_at: u64,
}
