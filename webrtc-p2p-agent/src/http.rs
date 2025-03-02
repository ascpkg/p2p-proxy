use serde_json;
use ureq;

use crate::data::{Agent, Configurations, Sdp};

pub fn publish_agent(config: &Configurations, agent: &Agent) {
    let url = format!(
        "{}{}/{}",
        config.signal_server_url, config.publish_agent_url, agent.name
    );

    let a = ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_read(std::time::Duration::from_secs(15))
        .timeout_write(std::time::Duration::from_secs(5))
        .build();

    let _response = a
        .post(&url)
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(agent).unwrap());
}

pub fn query_client_sdp(config: &Configurations, uuid: &str) -> Vec<Sdp> {
    let url = format!(
        "{}{}/{}",
        config.signal_server_url, config.query_client_sdp_url, uuid
    );

    let a = ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_read(std::time::Duration::from_secs(15))
        .timeout_write(std::time::Duration::from_secs(5))
        .build();

    if let Ok(response) = a.get(&url).call() {
        let body = response.into_string().unwrap();
        return serde_json::from_str(&body).unwrap();
    }

    return vec![];
}

pub fn publish_agent_sdp(config: &Configurations, uuid: &str, sdp: &Sdp) {
    let url = format!(
        "{}{}/{}",
        config.signal_server_url, config.publish_agent_sdp_url, uuid
    );

    let a = ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_read(std::time::Duration::from_secs(15))
        .timeout_write(std::time::Duration::from_secs(5))
        .build();

    let _response = a
        .post(&url)
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(sdp).unwrap());
}
