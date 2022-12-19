use std::{path::PathBuf, process::Command};

fn ask_user_input(path: &PathBuf) -> Result<String, String> {
    Command::new("sxiv")
        .args(["-t", "-o", &path.to_string_lossy()])
        .output()
        .or_else(|_| Err(format!("Failed to execute sxiv")))
        .and_then(|val| {
            String::from_utf8(val.stdout)
                .or_else(|_| Err("sxiv output is not valid utf8".to_string()))
        })
}

pub fn ask_user_input_single(path: &PathBuf) -> Result<PathBuf, String> {
    return ask_user_input(path)?
        .split('\n')
        .next()
        .and_then(|val| {
            if val.len() == 0 {
                None
            } else {
                Some(PathBuf::from(val))
            }
        })
        .ok_or_else(|| "No wallpaper selected.".to_string());
}
