use std::{
    path::Path,
    process::{Child, ChildStdout, Command, Stdio},
};

pub(super) struct FfmpegVideo {
    child: Child,
    pub(super) stdout: ChildStdout,
}

impl FfmpegVideo {
    pub(super) fn spawn_source(path: &Path, fps: u32) -> Result<Self, String> {
        if !path.is_file() {
            return Err(format!("video not found: {}", path.display()));
        }

        let filter = format!("fps={fps},format=rgb24");
        let mut child = Command::new("ffmpeg")
            .arg("-nostdin")
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-i")
            .arg(path)
            .arg("-an")
            .arg("-sn")
            .arg("-vf")
            .arg(filter)
            .arg("-f")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("rgb24")
            .arg("pipe:1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("failed to start ffmpeg: {error}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or("failed to capture ffmpeg stdout")?;
        Ok(Self { child, stdout })
    }
}

impl Drop for FfmpegVideo {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(super) struct AudioPlayback {
    child: Child,
}

impl AudioPlayback {
    pub(super) fn spawn(path: &Path) -> Result<Self, String> {
        spawn_audio_with("ffplay", path).or_else(|_| spawn_audio_with("afplay", path))
    }
}

fn spawn_audio_with(program: &str, path: &Path) -> Result<AudioPlayback, String> {
    let mut command = Command::new(program);
    if program == "ffplay" {
        command
            .arg("-nodisp")
            .arg("-autoexit")
            .arg("-loglevel")
            .arg("error")
            .arg("-i");
    }
    let child = command
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start {program}: {error}"))?;
    Ok(AudioPlayback { child })
}

impl Drop for AudioPlayback {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
