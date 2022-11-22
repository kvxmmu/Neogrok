use std::env;

static ENV_LOGLEVEL: &str = "NEOGROK_LOGLEVEL";

fn main() {
    if env::var(ENV_LOGLEVEL).is_err() {
        env::set_var(ENV_LOGLEVEL, "info");
    }
    pretty_env_logger::init_custom_env(ENV_LOGLEVEL);

    log::info!("Hello world");
}
