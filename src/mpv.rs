use std::os::unix::process::CommandExt;
use std::{path::PathBuf, process::Command};

pub fn run(socket_path: &PathBuf) {
    Command::new("xwinwrap")
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
}
