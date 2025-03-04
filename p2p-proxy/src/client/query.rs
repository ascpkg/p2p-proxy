use anyhow::Result;

use crate::{data::Configurations, http_client};

pub async fn process(name: &str) -> Result<()> {
    let config = Configurations::load_file(false);

    let agents = http_client::query_agent(&config, name);
    tracing::info!("agents: {:?}", agents);

    return Ok(());
}
