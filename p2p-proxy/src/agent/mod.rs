use std::collections::HashSet;

use anyhow::Result;
use tokio::{
    select,
    time::{sleep, Duration},
};
use tracing;

use crate::{
    aes::AesEncryption,
    candidate::IceEndpoint,
    data::{Agent, Configurations},
    http_client,
};

mod proxy;

pub async fn process() -> Result<()> {
    let config = Configurations::load_file(true);
    if config.signal_server_url.is_empty() {
        return Err(anyhow::anyhow!("config.signal_server_url.is_empty()"));
    }

    let agent = Agent {
        uuid: config.uuid.clone(),
        name: config.name.clone(),
        os: config.os.clone(),
    };
    http_client::publish_agent(&config, &agent);

    let mut last_local_ice_endpoint = None;
    let mut last_local_candidate_strings = String::new();
    let mut last_remote_candidate_strings = String::new();
    loop {
        select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("tokio::signal::ctrl_c()");

                http_client::delete_agent(&config, &agent);

                let remote_sdps = http_client::query_client_sdp(&config, &config.uuid);
                for sdp in remote_sdps {
                    http_client::delete_agent_sdp(&config, &config.uuid, &sdp);
                }

                break;
            }
            _ = async {
                let remote_sdps = http_client::query_client_sdp(&config, &config.uuid);

                let mut candidates = vec![String::new(); remote_sdps.len()];
                let mut decrypted_sdps = vec![String::new(); remote_sdps.len()];
                for i in 0..remote_sdps.len() {
                    match AesEncryption::new(&config.password).decrypt(&remote_sdps[i].sdp.as_slice()) {
                        Err(e) => {
                            tracing::error!("AesEncryption::new(&config.password).decrypt() error, e: {:?}", e);
                        }
                        Ok(text) => {
                            candidates[i] = IceEndpoint::to_unique_string(&text, remote_sdps[i].is_udp, remote_sdps[i].port)?;
                            decrypted_sdps[i] = text;
                        }
                    }
                }
                let remote_candidate_strings = candidates.join("\n");

                if remote_candidate_strings != last_remote_candidate_strings {
                    let local_ice_endpoint = IceEndpoint::collect(&config, 5).await?;
                    let local_candidate_strings = local_ice_endpoint.to_unique_strings().join("\n");
                    if local_candidate_strings != last_local_candidate_strings {
                        for i in 0..remote_sdps.len() {
                            match IceEndpoint::from_str(&decrypted_sdps[i], &config).await {
                                Err(e) => {
                                    tracing::error!("IceEndpoint::from_str() error, e: {:?}", e);
                                }
                                Ok(_remote_ice_endpoint) => {
                                    proxy::proxy(&config, &local_ice_endpoint, &remote_sdps[i]).await?;
                                }
                            }
                        }
                        let mut unique_ports = HashSet::new();
                        for c in &local_ice_endpoint.candidates {
                            // bind udp socket to local candidate
                            if unique_ports.contains(&c.port()) {
                                continue;
                            }
                            unique_ports.insert(c.port());
                            let local_address = format!("0.0.0.0:{}", c.port());
                            match tokio::net::UdpSocket::bind(&local_address).await {
                                Ok(_sock) => {

                                },
                                Err(e) => {
                                    tracing::error!(
                                        "tokio::net::UdpSocket::bind({}) error, e: {:?}",
                                        local_address,
                                        e
                                    );
                                }
                            };
                        }

                        last_local_ice_endpoint = Some(local_ice_endpoint);
                        last_local_candidate_strings = local_candidate_strings;
                    }
                    last_remote_candidate_strings = remote_candidate_strings;
                }

                sleep(Duration::from_secs(10)).await;

                Ok::<_, anyhow::Error>(())
            } => {}
        }
    }

    return Ok(());
}
