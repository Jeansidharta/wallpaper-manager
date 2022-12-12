use crate::utils::{make_error_message_after_command_call, trim_string};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn get_resolution(file_path: &PathBuf) -> Result<(i32, i32), String> {
    let resolution: String = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0",
            &file_path.to_string_lossy(),
        ])
        .stdout(Stdio::piped())
        .spawn()
        .or_else(|err| Err(make_error_message_after_command_call("ffprobe", err)))?
        .wait_with_output()
        .or_else(|_| Err("ffprobe failed".to_string()))?
        .stdout
        .into_iter()
        .map(|c| c as char)
        .collect::<String>();

    let mut split = resolution.split(',');
    let mut width_string: String = split
        .next()
        .ok_or_else(|| "After ffprobe split, failed to get width".to_string())?
        .to_string();
    let mut height_string: String = split
        .next()
        .ok_or_else(|| "After ffprobe split, failed to get height".to_string())?
        .to_string();
    trim_string(&mut width_string);
    trim_string(&mut height_string);
    let width = trim_string(&mut width_string)
        .parse()
        .or_else(|_| Err("Failed to parse width string".to_string()))?;
    let height = trim_string(&mut height_string)
        .parse()
        .or_else(|_| Err("Failed to parse height string".to_string()))?;
    Ok((width, height))
}

pub fn rescale_video(
    original_video_path: &PathBuf,
    new_width: i32,
    new_height: i32,
    new_video_path: &PathBuf,
) -> Result<(), String> {
    Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            &original_video_path.to_string_lossy(),
            "-vf",
            &format!("scale={}:{}", new_width, new_height),
            &new_video_path.to_string_lossy(),
        ])
        .spawn()
        .or_else(|err| Err(make_error_message_after_command_call("ffmpeg", err)))?
        .wait()
        .or_else(|_| Err("ffmpeg failed".to_string()))?;

    Ok(())
}

pub fn generate_thumbnail(
    original_video_path: &PathBuf,
    thumbnail_path: &PathBuf,
) -> Result<(), String> {
    Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            &original_video_path.to_string_lossy(),
            "-ss",
            "00:00:00.000",
            "-vframes",
            "1",
            "-vf",
            "scale=520:-1",
            &*thumbnail_path.to_string_lossy(),
        ])
        .spawn()
        .or_else(|err| Err(make_error_message_after_command_call("ffmpeg", err)))?
        .wait()
        .or_else(|_| Err("ffmpeg failed".to_string()))?;

    Ok(())
}
