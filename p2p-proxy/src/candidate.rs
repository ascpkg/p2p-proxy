use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;
use webrtc_ice::agent::{agent_config::AgentConfig, Agent};
use webrtc_ice::candidate::{candidate_base::unmarshal_candidate, Candidate};
use webrtc_ice::network_type::NetworkType;
use webrtc_ice::url::{ProtoType, Url};

use crate::data::Configurations;

static CANDIDATE_LINE_DELIMITER: &str = "\r\n";

pub struct IceEndpoint {
    agent: Agent,
    candidates: Vec<Arc<dyn Candidate + Send + Sync>>,
}

impl IceEndpoint {
    pub fn to_string(&self) -> String {
        return self
            .candidates
            .iter()
            .map(|c| c.marshal())
            .collect::<Vec<String>>()
            .join(CANDIDATE_LINE_DELIMITER);
    }

    pub async fn from_str(text: &str, config: &Configurations) -> Result<Self> {
        // setup ice agent config
        let ice_agent_config = AgentConfig {
            urls: config
                .stun_server_urls
                .iter()
                .map(|(is_udp, host, port)| Url {
                    scheme: webrtc_ice::url::SchemeType::Stun,
                    host: host.clone(),
                    port: port.clone(),
                    proto: if *is_udp {
                        ProtoType::Udp
                    } else {
                        ProtoType::Tcp
                    },
                    ..Default::default()
                })
                .collect(),
            network_types: vec![NetworkType::Udp4, NetworkType::Udp6],
            ..Default::default()
        };

        let agent = Agent::new(ice_agent_config).await?;

        let candidates = text
            .split(CANDIDATE_LINE_DELIMITER)
            .filter(|s| !s.is_empty())
            .map(|s| unmarshal_candidate(s))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|c| Arc::new(c) as Arc<dyn Candidate + Send + Sync>)
            .collect();

        return Ok(IceEndpoint {
            agent,
            candidates,
        });
    }

    pub async fn collect(config: &Configurations) -> Result<Self> {
        // setup ice agent config
        let ice_agent_config = AgentConfig {
            urls: config
                .stun_server_urls
                .iter()
                .map(|(is_udp, host, port)| Url {
                    scheme: webrtc_ice::url::SchemeType::Stun,
                    host: host.clone(),
                    port: port.clone(),
                    proto: if *is_udp {
                        ProtoType::Udp
                    } else {
                        ProtoType::Tcp
                    },
                    ..Default::default()
                })
                .collect(),
            network_types: vec![NetworkType::Udp4, NetworkType::Udp6],
            ..Default::default()
        };

        let ice_agent = Agent::new(ice_agent_config).await?;

        // notify candidate gathering is done
        let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);
        let done_tx = Arc::new(Mutex::new(Some(done_tx)));

        // collect candidates
        let ice_candidates = Arc::new(Mutex::new(Vec::new()));
        let ice_candidates_clone = Arc::clone(&ice_candidates);
        ice_agent.on_candidate(Box::new(move |c| {
            let candidates = Arc::clone(&ice_candidates_clone);
            let done_tx_clone = Arc::clone(&done_tx);
            Box::pin(async move {
                if let Some(c) = c {
                    let mut candidates = candidates.lock().await;
                    candidates.push(c);
                } else {
                    if let Some(tx) = done_tx_clone.lock().await.take() {
                        let _ = tx.send(()).await;
                    }
                }
            })
        }));

        // start candidate gathering
        ice_agent.gather_candidates()?;

        // wait for candidate gathering to complete
        let _ = done_rx.recv().await;

        let collected_ice_candidates = ice_candidates.lock().await.clone();

        Ok(IceEndpoint {
            agent: ice_agent,
            candidates: collected_ice_candidates,
        })
    }

    pub async fn test(
        &self,
        remote: &IceEndpoint,
    ) -> Result<
        Option<(
            Arc<dyn Candidate + Send + Sync>,
            Arc<dyn Candidate + Send + Sync>,
        )>,
    > {
        // sort candidates by priority
        let mut local_candidates = self.candidates.clone();
        local_candidates.sort_by(|a, b| b.priority().cmp(&a.priority()));

        let mut remote_candidates = remote.candidates.clone();
        remote_candidates.sort_by(|a, b| b.priority().cmp(&a.priority()));

        // test candidate pairs
        for local_candidate in &local_candidates {
            for remote_candidate in &remote_candidates {
                if Self::test_connectivity(
                    Arc::clone(local_candidate),
                    Arc::clone(remote_candidate),
                )
                .await
                {
                    return Ok(Some((
                        Arc::clone(local_candidate),
                        Arc::clone(remote_candidate),
                    )));
                }
            }
        }

        Ok(None)
    }

    async fn test_connectivity(
        local_candidate: Arc<dyn Candidate + Send + Sync>,
        remote_candidate: Arc<dyn Candidate + Send + Sync>,
    ) -> bool {
        // bind udp socket to local candidate
        let socket = match tokio::net::UdpSocket::bind(&local_candidate.address()).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    "tokio::net::UdpSocket::bind({}) error, e: {:?}",
                    local_candidate.address(),
                    e
                );
                return false;
            }
        };

        // send test data and wait for response
        let timeout = tokio::time::Duration::from_secs(1);
        let result = tokio::time::timeout(timeout, async {
            let test_data = b"ice-connectivity-test";
            match socket.send_to(test_data, &remote_candidate.address()).await {
                Ok(_n) => {}
                Err(e) => {
                    tracing::error!(
                        "socket.send_to({}) error, e: {:?}",
                        remote_candidate.address(),
                        e
                    );
                    return false;
                }
            }

            let mut buf = [0u8; 1024];
            match socket.recv_from(&mut buf).await {
                Ok((_n, _addr)) => {
                    return true;
                }
                Err(e) => {
                    tracing::error!("socket.recv_from() error, e: {:?}", e);
                    return false;
                }
            }
        })
        .await;

        return result.unwrap_or(false);
    }
}
