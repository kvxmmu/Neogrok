use {
    clap::{
        Parser,
        Subcommand,
    },
    std::num::NonZeroUsize,
};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create TCP server
    Tcp {
        /// Local address in format host:port, e.g.
        /// localhost:25565
        address: String,

        /// Port that will be bound on the server
        #[clap(long, short)]
        bind_port: Option<u16>,
    },
}

#[derive(Debug, Parser)]
pub struct Args {
    /// Neogrok server address
    #[clap(long, short)]
    pub remote: String,

    /// Number of worker threads, defaults to $(nproc)
    #[clap(long, short)]
    pub workers: Option<NonZeroUsize>,

    /// Authorization magic
    #[clap(long, short)]
    pub password: Option<String>,

    /// Action to perform
    #[clap(subcommand)]
    pub subcommand: Command,
}
