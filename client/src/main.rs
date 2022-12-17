use clap::Parser;
use neo::{
    args::Args,
    master::listener::run_listener,
};
use tokio::{
    net::TcpStream,
    runtime::Builder,
};
use tracing::Level;

async fn connect_and_start(args: Args) -> anyhow::Result<()> {
    let address = &args.remote;
    tracing::info!(%address, "Connecting...");
    let stream = TcpStream::connect(address).await?;

    run_listener(stream, args).await
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    let rt = match args
        .workers
        .map(|v| v.get())
        .unwrap_or_else(num_cpus::get)
    {
        1 => Builder::new_current_thread(),
        n => {
            let mut b = Builder::new_multi_thread();
            b.worker_threads(n);
            b
        }
    }
    .enable_io()
    .build()
    .expect("Failed to create tokio runtime");

    rt.block_on(connect_and_start(args))
}
