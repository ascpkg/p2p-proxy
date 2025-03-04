use serde_json;
use ureq;

use crate::data::{Agent, Configurations, Sdp};

static HTTP_WRITE_TIEOUT_SECS: u64 = 5;
static HTTP_READ_TIMEOUT_SECS: u64 = 15;
static HTTP_HEADER_KEY_CONTENT_TYPE: &str = "Content-Type";
static HTTP_HEADER_VALUE_APP_JSON: &str = "application/json";

pub fn publish_agent(config: &Configurations, agent: &Agent) {
    let url = format!(
        "{}{}/{}",
        config.signal_server_url, config.publish_agent_url, agent.name
    );

    let a = ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_read(std::time::Duration::from_secs(HTTP_READ_TIMEOUT_SECS))
        .timeout_write(std::time::Duration::from_secs(HTTP_WRITE_TIEOUT_SECS))
        .build();

    let _response = a
        .post(&url)
        .set(HTTP_HEADER_KEY_CONTENT_TYPE, HTTP_HEADER_VALUE_APP_JSON)
        .send_string(&serde_json::to_string(agent).unwrap());

    tracing::info!("publish_agent response: {:?}", _response);
}

pub fn query_agent(config: &Configurations, name: &str) -> Vec<Agent> {
    let url = format!(
        "{}{}/{}",
        config.signal_server_url, config.query_agent_url, name
    );

    let a = ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_read(std::time::Duration::from_secs(HTTP_READ_TIMEOUT_SECS))
        .timeout_write(std::time::Duration::from_secs(HTTP_WRITE_TIEOUT_SECS))
        .build();

    if let Ok(response) = a.get(&url).call() {
        let body = response.into_string().unwrap();
        return serde_json::from_str(&body).unwrap();
    }

    return vec![];
}

pub fn publish_client_sdp(config: &Configurations, uuid: &str, sdp: &Sdp) {
    let url = format!(
        "{}{}/{}",
        config.signal_server_url, config.publish_client_sdp_url, uuid
    );

    let a = ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_read(std::time::Duration::from_secs(HTTP_READ_TIMEOUT_SECS))
        .timeout_write(std::time::Duration::from_secs(HTTP_WRITE_TIEOUT_SECS))
        .build();

    let _response = a
        .post(&url)
        .set(HTTP_HEADER_KEY_CONTENT_TYPE, HTTP_HEADER_VALUE_APP_JSON)
        .send_string(&serde_json::to_string(sdp).unwrap());
}

pub fn query_client_sdp(config: &Configurations, uuid: &str) -> Vec<Sdp> {
    let url = format!(
        "{}{}/{}",
        config.signal_server_url, config.query_client_sdp_url, uuid
    );

    let a = ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_read(std::time::Duration::from_secs(HTTP_READ_TIMEOUT_SECS))
        .timeout_write(std::time::Duration::from_secs(HTTP_WRITE_TIEOUT_SECS))
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
        .timeout_read(std::time::Duration::from_secs(HTTP_READ_TIMEOUT_SECS))
        .timeout_write(std::time::Duration::from_secs(HTTP_WRITE_TIEOUT_SECS))
        .build();

    let _response = a
        .post(&url)
        .set(HTTP_HEADER_KEY_CONTENT_TYPE, HTTP_HEADER_VALUE_APP_JSON)
        .send_string(&serde_json::to_string(sdp).unwrap());
}

pub fn query_agent_sdp(config: &Configurations, uuid: &str) -> Vec<Sdp> {
    let url = format!(
        "{}{}/{}",
        config.signal_server_url, config.query_agent_sdp_url, uuid
    );

    let a = ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_read(std::time::Duration::from_secs(HTTP_READ_TIMEOUT_SECS))
        .timeout_write(std::time::Duration::from_secs(HTTP_WRITE_TIEOUT_SECS))
        .build();

    if let Ok(response) = a.get(&url).call() {
        let body = response.into_string().unwrap();
        return serde_json::from_str(&body).unwrap();
    }

    return vec![];
}
