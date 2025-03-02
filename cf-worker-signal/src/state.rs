use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use worker::kv::KvStore;

#[derive(Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Agent {
    pub uuid: String,
    pub name: String,
    pub os: String,
}

#[derive(Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Sdp {
    pub sdp: String,
    pub is_udp: bool,
    pub port: u16,
}

pub trait AbstractKvStore {
    fn get_kv_store_key() -> &'static str;
    fn get_expiration_ttl() -> u64;
    fn format_agent_key(name: &str) -> String;
    fn format_agent_sdp_key(uuid: &str) -> String;
    fn format_client_sdp_key(uuid: &str) -> String;
    fn format_service_key(is_udp: bool, port: u16) -> String;
}

pub struct AppStateKvStore {
    kv: KvStore,
}

impl AbstractKvStore for AppStateKvStore {
    fn get_kv_store_key() -> &'static str {
        "kv_cf_worker_signal"
    }

    fn get_expiration_ttl() -> u64 {
        3600
    }

    fn format_agent_key(name: &str) -> String {
        format!("agent:{}", name)
    }

    fn format_agent_sdp_key(uuid: &str) -> String {
        format!("agent_sdp:{}", uuid)
    }

    fn format_client_sdp_key(uuid: &str) -> String {
        format!("client_sdp:{}", uuid)
    }

    fn format_service_key(is_udp: bool, port: u16) -> String {
        format!("{}:{}", if is_udp { "udp" } else { "tcp" }, port)
    }
}

impl AppStateKvStore {
    pub fn new(kv: KvStore) -> Self {
        Self { kv }
    }

    pub async fn insert_or_update_agent(&mut self, name: &str, agent: Agent) {
        self.insert_or_update_generic(Self::format_agent_key(name), agent.uuid.clone(), agent)
            .await
    }

    pub async fn query_agent(&mut self, name: &str) -> Vec<Agent> {
        self.query_generic(Self::format_agent_key(name)).await
    }

    pub async fn insert_or_update_client_sdp(&mut self, uuid: &str, sdp: Sdp) {
        self.insert_or_update_generic(
            Self::format_client_sdp_key(uuid),
            Self::format_service_key(sdp.is_udp, sdp.port),
            sdp,
        )
        .await
    }

    pub async fn query_client_sdp(&mut self, uuid: &str) -> Vec<Sdp> {
        self.query_generic(Self::format_client_sdp_key(uuid)).await
    }

    pub async fn insert_or_update_agent_sdp(&mut self, uuid: &str, sdp: Sdp) {
        self.insert_or_update_generic(
            Self::format_agent_sdp_key(uuid),
            Self::format_service_key(sdp.is_udp, sdp.port),
            sdp,
        )
        .await
    }

    pub async fn query_agent_sdp(&mut self, uuid: &str) -> Vec<Sdp> {
        self.query_generic(Self::format_agent_sdp_key(uuid)).await
    }

    // Generic helper method for insert or update operations
    async fn insert_or_update_generic<T: Serialize + for<'de> Deserialize<'de> + PartialEq>(
        &mut self,
        key: String,
        sub_key: String,
        value: T,
    ) {
        let mut items: BTreeMap<String, T> = BTreeMap::new();
        if let Ok(option) = self.kv.get(&key).text().await {
            if let Some(text) = option {
                items = serde_json::from_str(&text).unwrap();
                if let Some(existing) = items.get(&sub_key) {
                    if existing == &value {
                        return;
                    }
                }
            }
        }

        items.insert(sub_key, value);
        self.kv
            .put(&key, serde_json::to_string(&items).unwrap())
            .unwrap()
            .expiration_ttl(Self::get_expiration_ttl())
            .execute()
            .await
            .unwrap();
    }

    // Generic helper method for query operations
    async fn query_generic<T: Clone + Serialize + for<'de> Deserialize<'de>>(
        &mut self,
        key: String,
    ) -> Vec<T> {
        if let Ok(option) = self.kv.get(&key).text().await {
            if let Some(text) = option {
                let items: BTreeMap<String, T> = serde_json::from_str(&text).unwrap();
                return items.values().cloned().collect();
            }
        }
        vec![]
    }
}
