use anyhow::{Context, Result};
use pollsendd::Runtime;

mod args;
mod pipewire;
mod service;
mod warmup;

use args::{Args, RunMode};
use service::PwModService;

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .format_target(false)
        .write_style(env_logger::WriteStyle::Always)
        .init();

    let args = Args::parse();
    log::trace!("Running with args {args:?}");

    std::thread::spawn(warmup::play_silence);

    let service = PwModService::new()?;
    let mut rt = match args.mode {
        RunMode::Systemd => Runtime::new_with_systemd_socket(service),
        RunMode::Dev => {
            let xdg_runtime_dir =
                std::env::var("XDG_RUNTIME_DIR").context("no $XDG_RUNTIME_DIR")?;
            let path = format!("{xdg_runtime_dir}/pw-mod-dev.sock");
            Runtime::new_with_socket_path(&path, service)
        }
    }?;

    loop {
        rt.poll()?;
    }
}
