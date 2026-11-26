mod config;
mod daemon;
mod input_relay;
mod measurement_relay;
mod sample_deadline;

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use config::{Config, DEFAULT_PROFILE_PATH};

fn main() -> ExitCode {
    match load_profile() {
        Ok(Some(profile)) => match start(profile) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("feeling-relay: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(None) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("feeling-relay: {error}");
            ExitCode::from(78)
        }
    }
}

fn load_profile() -> Result<Option<Config>, Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .map_or_else(|| PathBuf::from(DEFAULT_PROFILE_PATH), PathBuf::from);
    if args.next().is_some() {
        return Err("usage: feeling-relay [profile.toml]".into());
    }
    config::load(&path).map_err(|error| format!("{}: {error}", path.display()).into())
}

fn start(profile: Config) -> std::io::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?
        .block_on(daemon::run(profile))
}
