use std::process::Command;

#[cfg(debug_assertions)]
const SILENCE_WAV: &str = "data/silence.wav";

#[cfg(not(debug_assertions))]
const SILENCE_WAV: &str = "/usr/share/pipewire-mon/silence.wav";

pub(crate) fn play_silence() {
    log::info!("starting pipewire warmup");

    let output = match Command::new("/usr/bin/pw-play").arg(SILENCE_WAV).output() {
        Ok(output) => output,
        Err(err) => {
            log::error!("faile to warmup pipewie: {err:?}");
            std::process::exit(1);
        }
    };

    if !output.status.success() {
        log::error!("pw-play has exited with non-zero status code");

        let stdout = String::from_utf8_lossy(&output.stdout);
        log::error!("stdout: {stdout}");

        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("stderr: {stderr}");

        log::error!("failed to warmup pipewire");
        std::process::exit(1)
    }

    log::info!("finished pipewire warmup");
}
