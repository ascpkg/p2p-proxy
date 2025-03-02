use anyhow::Result;
use time::{macros::format_description, UtcOffset};

use tracing;
use tracing_subscriber::{self, fmt::time::OffsetTime};

pub mod data;
pub mod http;
pub mod rtc;
pub mod socket;
pub mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    // init stdout tracing log
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_line_number(true)
        .with_timer(OffsetTime::new(
            UtcOffset::from_hms(8, 0, 0).unwrap(),
            format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"),
        ))
        .init();

    let config = data::Configurations::load_file();
    if config.signal_server_url.is_empty() {
        return Err(anyhow::anyhow!("invalid config, missing signal_server_url"));
    }

    http::publish_agent(
        &config,
        &data::Agent {
            uuid: config.uuid.clone(),
            name: config.name.clone(),
            os: config.os.clone(),
        },
    );

    loop {
        let mut got = false;
        for client_sdp in http::query_client_sdp(&config, &config.uuid) {
            let (mut passive_done_tx, mut passive_done_rx) = tokio::sync::mpsc::channel(1);
            rtc::start_rtc_agent(&config, client_sdp, &mut passive_done_rx).await?;
            got = true;
        }
        if got {
            break;
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!();
        }
    };

    Ok(())
}
