use serde_json;
use ureq;

use crate::data::{Agent, Configurations, Sdp};

pub fn pub_agent(config: &Configurations, agent: &Agent) {
    let url = format!(
        "{}/{}/{}",
        config.signal_server_url, config.pub_agent_url, agent.name
    );

    let _response = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(agent).unwrap());
}

pub fn query_agent(config: &Configurations, name: &str) -> Vec<Agent> {
    let url = format!(
        "{}/{}/{}",
        config.signal_server_url, config.query_agent_url, name
    );

    if let Ok(response) = ureq::get(&url).call() {
        let body = response.into_string().unwrap();
        return serde_json::from_str(&body).unwrap();
    }

    return vec![];
}

pub fn pub_client_sdp(config: &Configurations, uuid: &str, sdp: &Sdp) {
    let url = format!(
        "{}/{}/{}",
        config.signal_server_url, config.pub_client_sdp_url, uuid
    );

    let _response = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(sdp).unwrap());
}

pub fn sub_client_sdp(config: &Configurations, uuid: &str) -> Vec<Sdp> {
    let url = format!(
        "{}/{}/{}",
        config.signal_server_url, config.sub_client_sdp_url, uuid
    );

    if let Ok(response) = ureq::get(&url).call() {
        let body = response.into_string().unwrap();
        return serde_json::from_str(&body).unwrap();
    }

    return vec![];
}

pub fn pub_agent_sdp(config: &Configurations, uuid: &str, sdp: &Sdp) {
    let url = format!(
        "{}/{}/{}",
        config.signal_server_url, config.pub_agent_sdp_url, uuid
    );

    let _response = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(sdp).unwrap());
}

pub fn sub_agent_sdp(config: &Configurations, uuid: &str) -> Vec<Sdp> {
    let url = format!(
        "{}/{}/{}",
        config.signal_server_url, config.sub_agent_sdp_url, uuid
    );

    if let Ok(response) = ureq::get(&url).call() {
        let body = response.into_string().unwrap();
        return serde_json::from_str(&body).unwrap();
    }

    return vec![];
}
