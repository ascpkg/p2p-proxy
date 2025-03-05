use std::str::FromStr;

use aes::AesEncryption;
use anyhow::Result;
use candidate::IceEndpoint;
use clap::Parser;
use data::Configurations;
use time::{macros::format_description, UtcOffset};
use tracing;
use tracing_subscriber::{self, fmt::time::OffsetTime};

mod aes;
mod agent;
mod candidate;
mod client;
mod command;
mod data;
mod http_client;

#[tokio::main]
async fn main() -> Result<()> {
    // parse command lines
    let cmd_args = command::Args::parse();

    // init tracing log to stdout
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&cmd_args.tracing_level).unwrap())
        .with_line_number(true)
        .with_timer(OffsetTime::new(
            UtcOffset::from_hms(8, 0, 0).unwrap(),
            format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"),
        ))
        .init();

    match cmd_args.command {
        command::Commands::Agent {} => agent::process().await?,
        command::Commands::Query { name } => client::query::process(&name).await?,
        command::Commands::Connect {
            name,
            uuid,
            udp,
            local_port,
            remote_port,
        } => client::connect::process(&name, &uuid, udp, local_port, remote_port).await?,
        command::Commands::Test { path, name, uuid } => {
            let config = Configurations::load_file(false);
            if path.starts_with(&config.query_client_sdp_url) {
                let sdps = http_client::query_client_sdp(&config, &uuid);
                for sdp in sdps {
                    match AesEncryption::new(&config.password).decrypt(&sdp.sdp.as_slice()) {
                        Err(e) => {
                            tracing::error!(
                                "AesEncryption::new(&config.password).decrypt() error, e: {:?}",
                                e
                            );
                        }
                        Ok(text) => {
                            tracing::info!(
                                "is_udp: {}, port: {}, candidates:\n{}",
                                sdp.is_udp,
                                sdp.port,
                                text
                            );
                        }
                    }
                }
            } else if path.starts_with(&config.query_agent_sdp_url) {
                let sdps = http_client::query_agent_sdp(&config, &uuid);
                for sdp in sdps {
                    match AesEncryption::new(&config.password).decrypt(&sdp.sdp.as_slice()) {
                        Err(e) => {
                            tracing::error!(
                                "AesEncryption::new(&config.password).decrypt() error, e: {:?}",
                                e
                            );
                        }
                        Ok(text) => {
                            tracing::info!(
                                "is_udp: {}, port: {}, candidates:\n{}",
                                sdp.is_udp,
                                sdp.port,
                                text
                            );
                        }
                    }
                }
            } else if path.starts_with(&config.query_agent_url) {
                let agents = http_client::query_agent(&config, &name);
                tracing::info!("{:?}", agents);
            }
        }
    }

    return Ok(());
}
