use std::{
    io,
    sync::Arc,
};

use neogrok::{
    config::Config,
    hisui::server::listen_hisui,
};
use tokio::runtime::Builder;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn load_cfg() -> Result<Config, Vec<&'static str>> {
    let paths = vec![
        "/etc/neogrok.toml",
        "/etc/neogrok/server.toml",
        "/etc/neogrok/neogrok.toml",
        "./neogrok.toml",
        "./config/neogrok.toml",
    ];

    for path in &paths {
        if let Ok(cfg) = Config::try_load_from(path) {
            return Ok(cfg);
        }
    }

    Err(paths)
}

fn main() -> io::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set default subscriber");
    let config: Arc<_> = match load_cfg() {
        Ok(cfg) => cfg,
        Err(tried_paths) => {
            tracing::error!(?tried_paths, "Failed to load config");
            std::process::exit(1);
        }
    }
    .into();

    let rt = match config.runtime.workers {
        1 | 0 => Builder::new_current_thread(),
        n => {
            let mut b = Builder::new_multi_thread();
            b.worker_threads(n);
            b
        }
    }
    .enable_io()
    .build()
    .expect("Failed to build tokio runtime");

    rt.block_on(listen_hisui(config))
}
