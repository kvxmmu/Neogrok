use clap::{
    Parser,
    Subcommand,
};

#[derive(Debug, Subcommand)]
pub enum NSub {
    /// Create TCP proxy
    Tcp {
        /// Local server address (e.g. 127.0.0.1:25565)
        address: String,

        /// Port that will be requested to bound on the
        /// remote server
        #[clap(long, short)]
        port: u16,
    },
}

#[derive(Debug, Parser)]
pub struct Args {
    /// Remote server authorization magic
    #[clap(long, short)]
    pub magic: Option<String>,

    /// Remote server address
    #[clap(long, short)]
    pub remote: String,

    /// Operation to perform
    #[clap(subcommand)]
    pub subcommand: NSub,
}
