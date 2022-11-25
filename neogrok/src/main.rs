use {
    clap::Parser,
    client::{
        args::Args,
        master::listener::listen_server,
    },
    std::env,
    tokio::{
        net::TcpStream,
        runtime::Builder,
    },
};

static ENV_LOGLEVEL: &str = "NEOGROK_LOGLEVEL";

async fn run(args: Args) -> anyhow::Result<()> {
    log::info!("Connecting to the {}...", args.remote);
    let stream = TcpStream::connect(&args.remote).await?;

    listen_server(stream, args)
        .await
        .map_err(|e| e.into())
}

fn main() -> anyhow::Result<()> {
    if env::var(ENV_LOGLEVEL).is_err() {
        env::set_var(ENV_LOGLEVEL, "info");
    }
    pretty_env_logger::init_custom_env(ENV_LOGLEVEL);

    let mut args = Args::parse();
    if !args.remote.contains(':') {
        args.remote.push_str(":6567");
    }

    let rt = match args.workers {
        Some(0 | 1) | None => Builder::new_current_thread(),
        Some(n) => {
            let mut b = Builder::new_multi_thread();
            b.worker_threads(n);
            b
        }
    }
    .thread_name("Neogrok client")
    .enable_io()
    .build()
    .expect("Failed to create tokio runtime");

    rt.block_on(run(args))
}
