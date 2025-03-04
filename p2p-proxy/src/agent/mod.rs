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

    // let mut last_local_candites = String::new();
    // let mut last_remote_signals = vec![];
    loop {
        select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("tokio::signal::ctrl_c()");
                break;
            }
            _ = async {
                let remote_sdps = http_client::query_client_sdp(&config, &config.uuid);
                if !remote_sdps.is_empty() {
                    let remote_sdp = &remote_sdps[0];
                    match AesEncryption::new(&config.password).decrypt(&remote_sdp.sdp.as_slice()) {
                        Err(e) => {
                            tracing::error!("AesEncryption::new().decrypt() error, e: {:?}", e);
                        }
                        Ok(sdp) => {
                            match IceEndpoint::from_str(&sdp, &config).await {
                                Err(e) => {
                                    tracing::error!("IceEndpoint::from_str() error, e: {:?}", e);
                                }
                                Ok(ice_endpoint) => {
                                    proxy::proxy(&config, &ice_endpoint, &remote_sdps).await?;
                                }
                            }
                        }
                    }
                }

                sleep(Duration::from_secs(10)).await;

                Ok::<_, anyhow::Error>(())
            } => {}
        }
    }

    return Ok(());
}
