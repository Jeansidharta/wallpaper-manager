use std::io::Write;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::{path::PathBuf, process::Command};

use crate::config::{ConfigOffset, ConfigResolution};

pub fn run(socket_path: &PathBuf, resolution: ConfigResolution, offset: ConfigOffset) {
    let mut command = Command::new("xwinwrap");

    command.args([
        "-ov",
        "-b",
        "-fs",
        "-ni",
        "-un",
        "-g",
        &format!(
            "{}x{}{:+}{:+}",
            resolution.width, resolution.height, offset.x, offset.y
        ),
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
    ]);
    println!("xwinwrap command: {:?}", command);
    command.exec();
}

pub fn load_file(socket_path: &PathBuf, image_path: &PathBuf) {
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
