use config_file_derives::ConfigFile;
use config_file_types;
use serde::{Deserialize, Serialize};

static CONFIG_PATH: &str = "agent.json";

#[derive(Debug, Default, Serialize, Deserialize, ConfigFile)]
#[config_file_ext("json")]
pub struct Configurations {
    #[serde(skip)]
    path: String,

    pub uuid: String,
    pub name: String,
    pub os: String,

    pub stun_server_urls: Vec<String>,

    pub signal_server_url: String,
    pub publish_agent_url: String,
    pub query_client_sdp_url: String,
    pub publish_agent_sdp_url: String,
}

impl Configurations {
    pub fn load_file() -> Self {
        let mut config = Self::load(CONFIG_PATH, true).unwrap();
        let mut update = false;
        if config.uuid.is_empty() {
            config.uuid = uuid::Uuid::new_v4().to_string();
            update = true;
        }
        if config.name.is_empty() {
            config.name = hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            update = true;
        }
        if config.os.is_empty() {
            config.os = std::env::consts::OS.to_string();
            update = true;
        }
        if config.stun_server_urls.is_empty() {
            config.stun_server_urls = vec!["stun:stun.l.google.com:19302".to_owned()];
            update = true;
        }
        if config.signal_server_url.is_empty() {
            tracing::error!("config.signal_server_url.is_empty()");
        }
        if config.publish_agent_url.is_empty() {
            config.publish_agent_url = String::from("/publish/agent");
            update = true;
        }
        if config.query_client_sdp_url.is_empty() {
            config.query_client_sdp_url = String::from("/query/client/sdp");
            update = true;
        }
        if config.publish_agent_sdp_url.is_empty() {
            config.publish_agent_sdp_url = String::from("/publish/agent/sdp");
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
