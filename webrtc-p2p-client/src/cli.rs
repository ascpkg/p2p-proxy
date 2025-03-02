use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// query agents with name filter
    Query {
        /// filter agents by name
        #[arg(long)]
        name: String,
    },
    /// connect to a specific agent by uuid
    Connect {
        /// filter agents by name
        #[arg(long)]
        name: String,

        /// the uuid of the agent to connect to
        #[arg(long, default_value = "")]
        uuid: String,

        /// whether to use tcp or udp
        #[arg(long)]
        udp: bool,

        /// the local port to listen on
        #[arg(long)]
        local_port: u16,

        /// the remote port to connect to
        #[arg(long)]
        remote_port: u16,
    },
}
