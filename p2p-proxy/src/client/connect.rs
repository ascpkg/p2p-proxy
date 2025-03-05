use anyhow::Result;
use tokio::{
    select,
    time::{sleep, Duration},
};

use crate::{
    aes::AesEncryption,
    candidate::IceEndpoint,
    data::{Configurations, Sdp},
    http_client,
};

pub async fn process(
    name: &str,
    uuid: &str,
    udp: bool,
    local_port: u16,
    remote_port: u16,
) -> Result<()> {
    let config = Configurations::load_file(false);

    let mut agents = http_client::query_agent(&config, name);
    if agents.is_empty() {
        let s = "agents.is_empty()";
        tracing::error!(s);
        return Err(anyhow::anyhow!(s));
    } else if agents.len() > 1 {
        if uuid.is_empty() {
            let s = "agents.len() > 1 && uuid.is_empty()";
            tracing::error!(s);
            return Err(anyhow::anyhow!(s));
        } else {
            agents = agents
                .into_iter()
                .filter(|agent| agent.uuid == uuid)
                .collect();
            if agents.is_empty() {
                let s =
                    "agents.into_iter().filter(|agent| agent.uuid == uuid).collect().is_empty()";
                tracing::error!(s);
                return Err(anyhow::anyhow!(s));
            }
        }
    }

    let local_ice_endpoint = IceEndpoint::collect(&config, 5).await?;
    let sdp = Sdp {
        is_udp: udp,
        port: remote_port,
        sdp: AesEncryption::new(&config.password).encrypt(&local_ice_endpoint.to_string())?,
    };
    http_client::publish_client_sdp(&config, uuid, &sdp);

    loop {
        select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("tokio::signal::ctrl_c()");

                http_client::delete_client_sdp(&config, uuid, &sdp);

                break;
            }
            _ = async {
                let sdps = http_client::query_agent_sdp(&config, uuid);
                if !sdps.is_empty() {
                    let sdp = &sdps[0];
                    let s = AesEncryption::new(&config.password).decrypt(&sdp.sdp.as_slice())?;
                    let remote_ice_endpoint = IceEndpoint::from_str(&s, &config).await?;
                    match local_ice_endpoint.test(&remote_ice_endpoint).await {
                        Err(e) => {
                            tracing::error!("local_ice_endpoint.test() error, e: {:?}", e);
                        }
                        Ok(None) => {
                            tracing::error!("local_ice_endpoint.test() error");
                        }
                        Ok(Some((local_candidate, remote_candidate))) => {
                            tracing::info!(
                                "local_candidate: {}, remote_candidate: {}",
                                local_candidate.address(),
                                remote_candidate.address()
                            );
                        }
                    }

                }
                sleep(Duration::from_secs(1)).await;

                Ok::<_, anyhow::Error>(())
            } => {}
        }
    }

    return Ok(());
}
