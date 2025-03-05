use sha1::{Digest, Sha1};

use config_file_derives::ConfigFile;
use config_file_types;
use serde::{Deserialize, Serialize};

static AGENT_CONFIG_PATH: &str = "agent.json";
static CLIENT_CONFIG_PATH: &str = "client.json";

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Agent {
    pub uuid: String,
    pub name: String,
    pub os: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Sdp {
    pub sdp: Vec<u8>,
    pub is_udp: bool,
    pub port: u16,
}

#[derive(Debug, Default, Serialize, Deserialize, ConfigFile)]
#[config_file_ext("json")]
pub struct Configurations {
    #[serde(skip)]
    path: String,

    pub password: String,

    pub uuid: String,
    pub name: String,
    pub os: String,

    pub stun_server_urls: Vec<(bool, String, u16)>,

    pub signal_server_url: String,
    pub publish_agent_url: String,
    pub query_agent_url: String,
    pub delete_agent_url: String,
    pub publish_client_sdp_url: String,
    pub query_client_sdp_url: String,
    pub delete_client_sdp_url: String,
    pub publish_agent_sdp_url: String,
    pub query_agent_sdp_url: String,
    pub delete_agent_sdp_url: String,
}

impl Configurations {
    pub fn load_file(is_agent: bool) -> Self {
        // load or create default config
        let mut config = Self::load(
            if is_agent {
                AGENT_CONFIG_PATH
            } else {
                CLIENT_CONFIG_PATH
            },
            true,
        )
        .unwrap();

        let mut update = false;
        if config.password.is_empty() {
            update = true;
            let mut hasher = sha1::Sha1::new();
            hasher.update(uuid::Uuid::new_v4().to_string().as_bytes());
            config.password = format!("{:x}", hasher.finalize());
        }
        if config.password.len() > 32 {
            update = true;
            config.password = config.password[..32].to_string();
        }
        if config.uuid.is_empty() {
            update = true;
            config.uuid = uuid::Uuid::new_v4().to_string();
        }
        if config.name.is_empty() {
            update = true;
            config.name = hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
        }
        if config.os.is_empty() {
            update = true;
            config.os = std::env::consts::OS.to_string();
        }
        if config.stun_server_urls.is_empty() {
            update = true;
            config.stun_server_urls = vec![(true, String::from("stun.l.google.com"), 19302)];
        }
        if config.signal_server_url.is_empty() {
            tracing::error!("config.signal_server_url.is_empty()");
        }
        if config.publish_agent_url.is_empty() {
            update = true;
            config.publish_agent_url = String::from("/publish/agent");
        }
        if config.query_agent_url.is_empty() {
            update = true;
            config.query_agent_url = String::from("/query/agent");
        }
        if config.delete_agent_url.is_empty() {
            update = true;
            config.delete_agent_url = String::from("/delete/agent");
        }
        if config.publish_client_sdp_url.is_empty() {
            update = true;
            config.publish_client_sdp_url = String::from("/publish/client/sdp");
        }
        if config.query_client_sdp_url.is_empty() {
            update = true;
            config.query_client_sdp_url = String::from("/query/client/sdp");
        }
        if config.delete_client_sdp_url.is_empty() {
            update = true;
            config.delete_client_sdp_url = String::from("/delete/client/sdp");
        }
        if config.publish_agent_sdp_url.is_empty() {
            update = true;
            config.publish_agent_sdp_url = String::from("/publish/agent/sdp");
        }
        if config.query_agent_sdp_url.is_empty() {
            update = true;
            config.query_agent_sdp_url = String::from("/query/agent/sdp");
        }
        if config.delete_agent_sdp_url.is_empty() {
            update = true;
            config.delete_agent_sdp_url = String::from("/delete/agent/sdp");
        }

        if update {
            config.dump(true, false);
        }

        return config;
    }
}
