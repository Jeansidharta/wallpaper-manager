use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::{io::Write, path::Path};

pub fn run(socket_path: &Path) -> anyhow::Result<()> {
    let exec = Command::new("xwinwrap")
        .args([
            "-ov",
            "-b",
            "-fs",
            "-g",
            "1920x1080+0+0",
            "--",
            "mpv",
            "-wid",
            "WID",
            "--idle=",
            "--no-osc",
            "--no-osd-bar",
            "--loop-file",
            "--player-operation-mode=cplayer",
            "--no-audio",
            "--panscan=1.0",
            "--no-input-default-bindings",
            &format!("--input-ipc-server={}", socket_path.to_string_lossy()),
        ])
        .exec();

    Err(exec)?
}

pub fn load_file(socket_path: &Path, image_path: &Path) {
    let mut socket_stream =
        UnixStream::connect(socket_path).expect("Failed to connect to MPV socket.");

    let payload = format!(
        "{{ \"command\": [\"loadfile\", \"{}\"] }}\n",
        image_path.to_string_lossy()
    );

    socket_stream
        .write_all(&payload.bytes().collect::<Vec<u8>>())
        .expect("Failed to write to MPV socket.");
}
