use anyhow::Result;

use crate::{
    candidate::IceEndpoint,
    data::{Configurations, Sdp},
};

pub async fn proxy(
    config: &Configurations,
    remote_ice_endpoint: &IceEndpoint,
    sdps: &Vec<Sdp>,
) -> Result<()> {
    let local_ice_endpoint = IceEndpoint::collect(config).await?;

    match local_ice_endpoint.test(remote_ice_endpoint).await {
        Err(e) => {
            tracing::error!("local_ice_endpoint.test() error, e: {:?}", e);
        }
        Ok(None) => {
            tracing::error!("local_ice_endpoint.test() error");
        }
        Ok(Some((local_candidate, remote_candidate))) => {}
    }

    return Ok(());
}
