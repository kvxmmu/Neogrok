use {
    config::{
        Config,
        LoggingConfig,
    },
    neogrok::tcp::listener::listen_to as listen_tcp,
    std::{
        io,
        process,
    },
    tokio::runtime::Builder,
};

fn handle_io_error(error: io::Error) -> ! {
    log::error!("Listener failed with {:#?}", error);

    process::exit(1);
}

fn main() {
    let config = Config::load("neogrok.toml");
    configure_logger(&config.logging);

    let rt = match config.server.workers {
        0 | 1 => Builder::new_current_thread(),
        n => {
            let mut rtb = Builder::new_multi_thread();
            rtb.worker_threads(n);
            rtb
        }
    }
    .enable_all()
    .build()
    .expect("Failed to create tokio runtime");

    rt.block_on(listen_tcp(config.into()))
        .map_err(handle_io_error)
        .unwrap();
}

fn configure_logger(cfg: &LoggingConfig) {
    let mut dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}]:{} {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(cfg.level.into_log())
        .chain(std::io::stderr());
    for file in &cfg.files {
        dispatch = dispatch
            .chain(fern::log_file(file).expect("Failed to create log file"));
    }

    dispatch.apply().expect("Failed to setup logging");
}
