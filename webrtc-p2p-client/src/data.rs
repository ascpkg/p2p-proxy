use config_file_derives::ConfigFile;
use config_file_types;
use serde::{Deserialize, Serialize};
use tracing;

static CONFIG_PATH: &str = "client.json";

#[derive(Debug, Default, Serialize, Deserialize, ConfigFile)]
#[config_file_ext("json")]
pub struct Configurations {
    #[serde(skip)]
    path: String,

    pub stun_server_urls: Vec<String>,

    pub signal_server_url: String,
    pub query_agent_url: String,
    pub publish_client_sdp_url: String,
    pub query_agent_sdp_url: String,
}

impl Configurations {
    pub fn load_file() -> Self {
        let mut config = Self::load(CONFIG_PATH, true).unwrap();
        let mut update = false;
        if config.stun_server_urls.is_empty() {
            config.stun_server_urls = vec!["stun:stun.l.google.com:19302".to_owned()];
            update = true;
        }
        if config.signal_server_url.is_empty() {
            tracing::error!("config.signal_server_url.is_empty()");
        }
        if config.query_agent_url.is_empty() {
            config.query_agent_url = String::from("/query/agent");
            update = true;
        }
        if config.publish_client_sdp_url.is_empty() {
            config.publish_client_sdp_url = String::from("/publish/client/sdp");
            update = true;
        }
        if config.query_agent_sdp_url.is_empty() {
            config.query_agent_sdp_url = String::from("/query/agent/sdp");
            update = true;
        }
        if update {
            config.dump(true, false);
        }
        config
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Agent {
    pub uuid: String,
    pub name: String,
    pub os: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Sdp {
    pub sdp: String,
    pub is_udp: bool,
    pub port: u16,
}
