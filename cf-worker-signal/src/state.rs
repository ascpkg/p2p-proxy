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

pub struct AppState {}

impl AppState {
    pub fn get_kv_store_key() -> &'static str {
        "kv_cf_worker_signal"
    }

    pub fn get_expiration_ttl() -> u64 {
        3600
    }

    pub fn format_agent_key(name: &str) -> String {
        format!("agent:{}", name)
    }

    pub fn format_agent_sdp_key(uuid: &str) -> String {
        format!("agent_sdp:{}", uuid)
    }

    pub fn format_client_sdp_key(uuid: &str) -> String {
        format!("client_sdp:{}", uuid)
    }

    pub fn format_service_key(is_udp: bool, port: u16) -> String {
        format!("{}:{}", if is_udp { "udp" } else { "tcp" }, port)
    }

    pub async fn insert_or_update_agent(kv: &mut KvStore, name: &str, agent: Agent) {
        let key = Self::format_agent_key(name);

        let mut agents: BTreeMap<String, Agent> = BTreeMap::new();
        if let Ok(o) = kv.get(&key).text().await {
            if let Some(s) = o {
                agents = serde_json::from_str(&s).unwrap();
                if let Some(ss) = agents.get_mut(&agent.uuid) {
                    if ss == &agent {
                        // skip
                        return;
                    }
                }
            }
        }

        // insert or update
        agents.insert(agent.uuid.clone(), agent);
        kv.put(&key, serde_json::to_string(&agents).unwrap())
            .unwrap()
            .expiration_ttl(Self::get_expiration_ttl())
            .execute()
            .await
            .unwrap();
    }

    pub async fn query_agent(kv: &mut KvStore, name: &str) -> Vec<Agent> {
        let key = Self::format_agent_key(name);

        if let Ok(o) = kv.get(&key).text().await {
            if let Some(s) = o {
                let agents: BTreeMap<String, Agent> = serde_json::from_str(&s).unwrap();
                return agents.values().cloned().collect();
            }
        }

        return vec![];
    }

    pub async fn insert_or_update_client_sdp(kv: &mut KvStore, uuid: &str, sdp: Sdp) {
        let key = Self::format_client_sdp_key(uuid);
        let sub_key = Self::format_service_key(sdp.is_udp, sdp.port);

        let mut sdps: BTreeMap<String, Sdp> = BTreeMap::new();
        if let Ok(o) = kv.get(&key).text().await {
            if let Some(s) = o {
                sdps = serde_json::from_str(&s).unwrap();
                if let Some(ss) = sdps.get_mut(&sub_key) {
                    if ss == &sdp {
                        // skip
                        return;
                    }
                }
            }
        }

        // insert or update
        sdps.insert(sub_key, sdp);
        kv.put(&key, serde_json::to_string(&sdps).unwrap())
            .unwrap()
            .expiration_ttl(Self::get_expiration_ttl())
            .execute()
            .await
            .unwrap();
    }

    pub async fn query_client_sdp(kv: &mut KvStore, uuid: &str) -> Vec<Sdp> {
        let key = Self::format_client_sdp_key(uuid);
        if let Ok(o) = kv.get(&key).text().await {
            if let Some(s) = o {
                let sdps: BTreeMap<String, Sdp> = serde_json::from_str(&s).unwrap();
                return sdps.values().cloned().collect();
            }
        }

        return vec![];
    }

    pub async fn insert_or_update_agent_sdp(kv: &mut KvStore, uuid: &str, sdp: Sdp) {
        let key = Self::format_agent_sdp_key(uuid);
        let sub_key = Self::format_service_key(sdp.is_udp, sdp.port);

        let mut sdps: BTreeMap<String, Sdp> = BTreeMap::new();
        if let Ok(o) = kv.get(&key).text().await {
            if let Some(s) = o {
                sdps = serde_json::from_str(&s).unwrap();
                if let Some(ss) = sdps.get_mut(&sub_key) {
                    if ss == &sdp {
                        // skip
                        return;
                    }
                }
            }
        }

        // insert or update
        sdps.insert(sub_key, sdp);
        kv.put(&key, serde_json::to_string(&sdps).unwrap())
            .unwrap()
            .expiration_ttl(Self::get_expiration_ttl())
            .execute()
            .await
            .unwrap();
    }

    pub async fn query_agent_sdp(kv: &mut KvStore, uuid: &str) -> Vec<Sdp> {
        let key = Self::format_agent_sdp_key(uuid);
        if let Ok(o) = kv.get(&key).text().await {
            if let Some(s) = o {
                let sdps: BTreeMap<String, Sdp> = serde_json::from_str(&s).unwrap();
                return sdps.values().cloned().collect();
            }
        }

        return vec![];
    }
}
