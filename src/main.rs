use clap::{Parser, Subcommand};
use config::read_config;
use std::collections::hash_map::HashMap;
use std::fs::{create_dir_all, read_dir, remove_file};
use std::io::{ErrorKind, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::exit;
use std::process::Command;

mod config;

fn generate_thumbnails(wallpapers_dir: &PathBuf, cache_dir: &PathBuf) {
    let mut cached_filenames = HashMap::new();

    println!("Listing wallpapers at {:?}", wallpapers_dir);
    for dir_item in read_dir(wallpapers_dir).expect("Could not read wallpapers directory.") {
        let dir_item = dir_item.expect("Failed to unwrap directory");
        let file_type = dir_item
            .file_type()
            .expect("Failed to get filetype from dir_item");

        if !file_type.is_file() {
            // Skip non-files
            continue;
        };

        let file_path = dir_item.path();
        let file_name = file_path
            .file_name()
            .expect("Failed to extract file name from path")
            .to_string_lossy();
        let file_stem = file_path
            .file_stem()
            .expect("Could not extract file stem.")
            .to_string_lossy();
        let cache_path = cache_dir.join(format!("{}.jpg", file_stem));

        cached_filenames.insert(cache_path.clone(), true);

        if cache_path.exists() {
            println!(
                "Cache for file {} already exists",
                file_path.file_name().unwrap().to_string_lossy()
            );
            continue;
        }

        println!("Missing cache for file {}. Generating...", file_name,);
        Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-i",
                &file_path.to_string_lossy(),
                "-ss",
                "00:00:00.000",
                "-vframes",
                "1",
                &*cache_path.to_string_lossy(),
            ])
            .spawn()
            .unwrap_or_else(|err| match err.kind() {
                ErrorKind::NotFound => {
                    println!("ffmpeg binary not found");
                    exit(-1);
                }
                _ => {
                    println!("Could not start ffmpeg");
                    exit(-1);
                }
            })
            .wait()
            .expect("ffmpeg failed");
    }

    for dir_item in read_dir(cache_dir).expect("Could not read cache directory") {
        let dir_item = dir_item.expect("Failed to unwrap directory");
        let file_type = dir_item
            .file_type()
            .expect("Failed to get filetype from dir_item");

        if !file_type.is_file() {
            // Skip non-files
            continue;
        };

        let file_path = dir_item.path();

        // There is a wallpaper for this cache item
        if cached_filenames.contains_key(&file_path) {
            continue;
        }

        println!(
            "Cache item {} has no wallpaper. Removing it.",
            file_path.to_string_lossy()
        );

        remove_file(&file_path).expect("Failed to remove cached item");
    }
}

fn daemonize(socket_path: &PathBuf) {
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

fn select_wallpaper(wallpapers_dir: &PathBuf, cache_dir: &PathBuf, socket_path: &PathBuf) {
    let sxiv_stdout = Command::new("sxiv")
        .args(["-t", "-o", &cache_dir.to_string_lossy()])
        .output()
        .expect("Failed to execute sxiv")
        .stdout;

    let sxiv_stdout = String::from_utf8(sxiv_stdout).expect("sxiv output is not valid utf8");

    let selected_wallpaper = sxiv_stdout
        .split('\n')
        .next()
        .expect("No wallpaper selected.");

    let selected_path = PathBuf::from(selected_wallpaper);
    let selected_file_name = selected_path
        .file_name()
        .expect("Selected wallpaper has no name");

    println!("Selected \"{}\"", selected_file_name.to_string_lossy());
    let selected_file_stem = selected_path
        .file_stem()
        .expect("Selected wallpaper has no stem");
    for dir_entry in read_dir(wallpapers_dir).expect("Failed to read wallpapers dir") {
        let dir_entry = dir_entry.expect("Wallpaper dir entry failed for an unexpected reason.");
        let dir_entry_path = dir_entry.path();
        let dir_entry_stem = dir_entry_path
            .file_stem()
            .expect("Failedto get wallpaper dir entry file stem");
        if dir_entry_stem == selected_file_stem {
            let mut socket_stream =
                UnixStream::connect(socket_path).expect("Failed to connect to MPV socket.");

            let payload = format!(
                "{{ \"command\": [\"loadfile\", \"{}\"] }}\n",
                dir_entry_path.to_string_lossy()
            );

            socket_stream
                .write_all(&payload.bytes().collect::<Vec<u8>>())
                .expect("Failed to write to MPV socket.");
            return;
        };
    }

    panic!("Failed to find a wallpaper that corresponds to the cached file. Is the cache stale?");
}

/// Program to manage my personal wallpapers
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory to look for the wallpapers
    #[arg(short, long)]
    wallpapers_dir: Option<PathBuf>,

    /// Where thumbnails are stored
    #[arg(short = 'e', long)]
    cache_dir: Option<PathBuf>,

    /// Where the configuration can be found
    #[arg(short, long)]
    config_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Daemon {
        // The path to the MPV socket. Defaults to /tmp/wallpaper-mpv-socket
        #[arg(short, long, default_value = "/tmp/wallpaper-mpv-socket")]
        socket_path: PathBuf,
    },
    GenerateThumbnails {},
    SelectWallpaper {
        // The path to the MPV socket. Defaults to /tmp/wallpaper-mpv-socket
        #[arg(short, long, default_value = "/tmp/wallpaper-mpv-socket")]
        socket_path: PathBuf,
    },
}

fn main() {
    let args = Args::parse();

    let config = read_config(args.config_dir).unwrap();

    let cache_dir = args.cache_dir.or(config.cache_dir).expect(&format!("Could not resolve the cache directory. Provide it in the configuration file or through --cache-dir"));

    if !cache_dir.is_dir() {
        create_dir_all(&cache_dir).expect("Failed to create cache directory");
    }

    let wallpapers_dir = args
        .wallpapers_dir
        .or(config.wallpapers_dir)
        .expect(&format!("Could not resolve the wallpapers directory. Provide it in the configuration file or through --cache-dir"));

    if !wallpapers_dir.is_dir() {
        panic!("Wallpapers directory does not exist");
    }

    match &args.command {
        Commands::Daemon { socket_path } => daemonize(&socket_path),
        Commands::GenerateThumbnails {} => generate_thumbnails(&wallpapers_dir, &cache_dir),
        Commands::SelectWallpaper { socket_path } => {
            select_wallpaper(&wallpapers_dir, &cache_dir, &socket_path)
        }
    }
}
