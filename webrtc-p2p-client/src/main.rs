use anyhow::Result;
use time::{macros::format_description, UtcOffset};

use tracing;
use tracing_subscriber::{self, fmt::time::OffsetTime};

pub mod cli;
pub mod data;
pub mod http;
pub mod rtc;
pub mod utils;

use clap::Parser;
use cli::{Cli, Commands};

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

    let cli = Cli::parse();

    let config = data::Configurations::load_file();
    if config.signal_server_url.is_empty() {
        return Err(anyhow::anyhow!("invalid config, missing signal_server_url"));
    }

    match cli.command {
        Commands::Query { name } => {
            let agents = http::query_agent(&config, &name);
            println!("agents: {:?}", agents);
        }
        Commands::Connect {
            name,
            uuid,
            udp,
            local_port,
            remote_port,
        } => {
            let agents = http::query_agent(&config, &name);
            if agents.is_empty() {
                tracing::error!("no agents found for name: {}", name);
            } else {
                let agent = if agents.len() == 1 {
                    agents[0].clone()
                } else {
                    agents
                        .iter()
                        .filter(|agent| agent.uuid == uuid)
                        .next()
                        .unwrap()
                        .clone()
                };
                rtc::start_rtc_client(&config, agent, udp, local_port, remote_port).await?;
            }
        }
    }

    Ok(())
}
