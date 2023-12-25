use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn ask_user_input(path: &Path) -> Result<String, String> {
    Command::new("sxiv")
        .args(["-t", "-o", &path.to_string_lossy()])
        .output()
        .map_err(|_| "Failed to execute sxiv".to_string())
        .and_then(|val| {
            String::from_utf8(val.stdout).map_err(|_| "sxiv output is not valid utf8".to_string())
        })
}

pub fn ask_user_input_single(path: &Path) -> Result<PathBuf, String> {
    return ask_user_input(path)?
        .split('\n')
        .next()
        .and_then(|val| {
            if val.is_empty() {
                None
            } else {
                Some(PathBuf::from(val))
            }
        })
        .ok_or_else(|| "No wallpaper selected.".to_string());
}
