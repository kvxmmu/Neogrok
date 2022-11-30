use {
    clap::{
        Parser,
        Subcommand,
    },
    std::num::NonZeroUsize,
};

#[derive(Debug, Subcommand)]
pub enum Command {
    Tcp {
        address: String,

        #[clap(long, short)]
        bind_port: Option<u16>,
    },
}

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long, short)]
    pub remote: String,

    #[clap(long, short)]
    pub workers: Option<NonZeroUsize>,

    #[clap(long, short)]
    pub password: Option<String>,

    #[clap(subcommand)]
    pub subcommand: Command,
}
