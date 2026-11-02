use anyhow::Result;
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
        RunMode::Dev => Runtime::new_with_socket_path("pw-mod-dev.sock", service),
    }?;

    loop {
        rt.poll()?;
    }
}
