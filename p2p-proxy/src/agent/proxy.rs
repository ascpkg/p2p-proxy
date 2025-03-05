use anyhow::Result;

use crate::{
    aes::AesEncryption,
    candidate::IceEndpoint,
    data::{Configurations, Sdp},
    http_client,
};

pub async fn proxy(
    config: &Configurations,
    local_ice_endpoint: &IceEndpoint,
    remote_sdp: &Sdp,
) -> Result<()> {
    let cipher_sdp =
        AesEncryption::new(&config.password).encrypt(&local_ice_endpoint.to_string())?;

    let mut local_sdp = remote_sdp.clone();
    local_sdp.sdp = cipher_sdp.clone();
    http_client::publish_agent_sdp(config, &config.uuid, &local_sdp);

    return Ok(());
}
