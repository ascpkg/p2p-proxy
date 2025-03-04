use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// set the maximum level that will be enabled by the tracing subscriber
    #[arg(long, default_value = "info")]
    pub tracing_level: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// run agent
    Agent {},

    /// query agents with name filter
    Query {
        /// filter agents by name
        #[arg(long)]
        name: String,
    },

    /// connect to a specific agent with name and uuid
    Connect {
        /// filter agents by name
        #[arg(long)]
        name: String,

        /// the uuid of the agent to connect to
        #[arg(long, default_value = "")]
        uuid: String,

        /// whether to use tcp or udp
        #[arg(long, default_value_t = false)]
        udp: bool,

        /// the local port to listen on
        #[arg(long)]
        local_port: u16,

        /// the remote port to connect to
        #[arg(long)]
        remote_port: u16,
    },
}
