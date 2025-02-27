use config_file_derives::ConfigFile;
use config_file_types;
use serde::{Deserialize, Serialize};

static CONFIG_PATH: &str = "client.json";

#[derive(Debug, Default, Serialize, Deserialize, ConfigFile)]
#[config_file_ext("json")]
pub struct Configurations {
    #[serde(skip)]
    path: String,

    pub signal_server_url: String,
    pub pub_agent_url: String,
    pub query_agent_url: String,
    pub pub_client_sdp_url: String,
    pub sub_client_sdp_url: String,
    pub pub_agent_sdp_url: String,
    pub sub_agent_sdp_url: String,
}

impl Configurations {
    pub fn load_file() -> Self {
        let mut config = Self::load(CONFIG_PATH, true).unwrap();
        let mut update = false;
        if config.pub_agent_url.is_empty() {
            config.pub_agent_url = String::from("/pub/agent");
            update = true;
        }
        if config.query_agent_url.is_empty() {
            config.query_agent_url = String::from("/query/agent");
            update = true;
        }
        if config.pub_client_sdp_url.is_empty() {
            config.pub_client_sdp_url = String::from("/pub/client/sdp");
            update = true;
        }
        if config.sub_client_sdp_url.is_empty() {
            config.sub_client_sdp_url = String::from("/sub/client/sdp");
            update = true;
        }
        if config.pub_agent_sdp_url.is_empty() {
            config.pub_agent_sdp_url = String::from("/pub/agent/sdp");
            update = true;
        }
        if config.sub_agent_sdp_url.is_empty() {
            config.sub_agent_sdp_url = String::from("/sub/agent/sdp");
            update = true;
        }
        if update {
            config.dump(true, false);
        }
        config
    }
}

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
