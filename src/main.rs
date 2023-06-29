use clap::{Parser, Subcommand};
use config::{read_config, ConfigResolution};
use std::collections::hash_map::HashMap;
use std::fs::{create_dir_all, read_dir, remove_file};
use std::path::PathBuf;

use crate::ffmpeg::{generate_thumbnail, get_resolution, rescale_video};

mod config;
mod ffmpeg;
mod mpv;
mod sxiv;
mod utils;

trait PrintableError<T> {
    fn print_and_exit(self) -> T;
}

impl<T> PrintableError<T> for Result<T, String> {
    fn print_and_exit(self) -> T {
        self.unwrap_or_else(|s| {
            println!("{}", s);
            std::process::exit(-1);
        })
    }
}

fn try_generate_rescaled_wallpaper(
    config_resolution: &ConfigResolution,
    file_path: &PathBuf,
    file_name: &str,
    rescaled_path: &PathBuf,
) {
    let (width, height) = get_resolution(file_path).print_and_exit();
    if width == config_resolution.width && height == config_resolution.height {
        return;
    }
    if rescaled_path.is_file() {
        return;
    }
    println!(
        "Resolution {}x{} does not match for {}. Generating rescaled version...",
        width, height, file_name,
    );
    rescale_video(
        file_path,
        config_resolution.width,
        config_resolution.height,
        &rescaled_path,
    )
    .print_and_exit();
    println!("Rescaled version generated!");
}

fn try_generate_thumbnail_for_wallpaper(
    file_path: &PathBuf,
    file_name: &str,
    thumbnail_path: &PathBuf,
) -> bool {
    if thumbnail_path.exists() {
        println!("Thumbnail for file {} already exists", file_name);
        return false;
    }

    println!("Missing thumbnail for file {}. Generating...", file_name,);
    generate_thumbnail(file_path, &thumbnail_path).print_and_exit();
    true
}

fn generate_cache(
    config_resolution: &ConfigResolution,
    wallpapers_dir: &PathBuf,
    thumbnails_cache_dir: &PathBuf,
    wallpapers_rescaled_dir: &PathBuf,
) {
    let mut cached_filenames = HashMap::new();
    let mut rescaled_wallpapers = HashMap::new();

    println!("Listing wallpapers at {:?}", wallpapers_dir);
    for dir_item in read_dir(wallpapers_dir).expect("Could not read wallpapers directory.") {
        let dir_item = dir_item.expect("Failed to unwrap directory");
        // Skip non-files
        {
            let file_type = dir_item
                .file_type()
                .expect("Failed to get filetype from dir_item");

            if !file_type.is_file() {
                continue;
            };
        }

        let file_path = dir_item.path();
        let file_stem = file_path
            .file_stem()
            .expect("Could not extract file stem.")
            .to_string_lossy();
        let file_name = file_path
            .file_name()
            .expect("Could not extract file name.")
            .to_string_lossy();

        let thumbnail_path = thumbnails_cache_dir.join(format!("{}.jpg", file_stem));
        let rescaled_path = wallpapers_rescaled_dir.join(file_name.to_string());

        try_generate_thumbnail_for_wallpaper(&file_path, &file_name, &thumbnail_path);
        try_generate_rescaled_wallpaper(config_resolution, &file_path, &file_name, &rescaled_path);

        cached_filenames.insert(thumbnail_path, true);
        rescaled_wallpapers.insert(rescaled_path, true);
    }

    let remove_unused_cache_files =
        |path: &PathBuf,
         dict: &HashMap<PathBuf, bool>,
         remove_message_maker: fn(path: &PathBuf)| {
            for dir_item in read_dir(path).expect("Could not read cache directory") {
                let dir_item = dir_item.expect("Failed to unwrap directory");
                // Skip non-files
                {
                    let file_type = dir_item
                        .file_type()
                        .expect("Failed to get filetype from dir_item");

                    if !file_type.is_file() {
                        continue;
                    };
                }

                let file_path = dir_item.path();

                // There is a wallpaper for this cache item
                if dict.contains_key(&file_path) {
                    continue;
                }

                remove_message_maker(&file_path);

                remove_file(&file_path).expect("Failed to remove cached item");
            }
        };
    remove_unused_cache_files(thumbnails_cache_dir, &cached_filenames, |path| {
        println!(
            "Thumbnail named {} has no wallpaper. Removing it.",
            path.to_string_lossy()
        )
    });
    remove_unused_cache_files(wallpapers_rescaled_dir, &rescaled_wallpapers, |path| {
        println!(
            "Rescaled wallpaper named {} has no wallpaper. Removing it.",
            path.to_string_lossy()
        )
    });
}

fn select_wallpaper(
    wallpapers_dir: &PathBuf,
    wallpapers_rescaled_dir: &PathBuf,
    thumbnails_cache_dir: &PathBuf,
    socket_path: &PathBuf,
    is_static: bool,
) {
    println!("{}", thumbnails_cache_dir.to_string_lossy());
    let selected_path = sxiv::ask_user_input_single(if is_static {
        wallpapers_dir
    } else {
        thumbnails_cache_dir
    })
    .print_and_exit();

    let selected_file_name = selected_path
        .file_name()
        .ok_or_else(|| "Selected wallpaper has no name".to_string())
        .print_and_exit();

    println!("Selected \"{}\"", selected_file_name.to_string_lossy());
    let selected_file_stem = selected_path
        .file_stem()
        .expect("Selected wallpaper has no stem");

    let find_wallpaper_path = |dir: &PathBuf| -> Option<PathBuf> {
        for dir_entry in read_dir(dir).expect("Failed to read wallpapers dir") {
            let dir_entry_path = dir_entry
                .expect("Wallpaper dir entry failed for an unexpected reason.")
                .path();
            let dir_entry_stem = dir_entry_path
                .file_stem()
                .expect("Failed to get wallpaper dir entry file stem");
            if dir_entry_stem == selected_file_stem {
                return Some(dir_entry_path);
            };
        }
        None
    };

    let path = find_wallpaper_path(wallpapers_rescaled_dir)
        .or_else(|| find_wallpaper_path(wallpapers_dir))
        .expect("Could not find corresponding wallpaper");

    mpv::load_file(socket_path, &path)
}

/// Program to manage my personal wallpapers
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory to look for the wallpapers
    #[arg(short, long)]
    wallpapers_dir: Option<PathBuf>,

    /// Where tciwhumbnails and resaized wallpapers are stored
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
    GenerateCache {},
    SelectWallpaper {
        // The path to the MPV socket. Defaults to /tmp/wallpaper-mpv-socket
        #[arg(short, long, default_value = "/tmp/wallpaper-mpv-socket")]
        socket_path: PathBuf,

        // If provided, will atempt to load the result as a static image.
        #[arg(short = 'c', long = "static")]
        static_image: bool,
    },
}

fn main() {
    let args = Args::parse();

    let config = read_config(args.config_dir).print_and_exit();

    let cache_dir = args.cache_dir
        .or(config.cache_dir)
        .expect(&format!("Could not resolve the cache directory. Provide it in the configuration file or through --cache-dir"));

    let wallpapers_dir = args
        .wallpapers_dir
        .or(config.wallpapers_dir)
        .expect(&format!("Could not resolve the wallpapers directory. Provide it in the configuration file or through --cache-dir"));

    let thumbnails_cache_dir = cache_dir.join("wallpapers-thumbnail");
    let wallpapers_rescaled_dir = cache_dir.join("wallpapers-rescaled");

    if !thumbnails_cache_dir.is_dir() {
        create_dir_all(&thumbnails_cache_dir).expect("Failed to create thumbnails cache directory");
    }
    if !wallpapers_rescaled_dir.is_dir() {
        create_dir_all(&wallpapers_rescaled_dir)
            .expect("Failed to create wallpapers rescaled cache directory");
    }

    if !wallpapers_dir.is_dir() {
        panic!("Wallpapers directory does not exist");
    }

    match &args.command {
        Commands::Daemon { socket_path } => mpv::run(
            &socket_path,
            config.resolution.unwrap_or_default(),
            config.offset.unwrap_or_default(),
        ),
        Commands::GenerateCache {} => generate_cache(
            &config.resolution.unwrap_or(ConfigResolution::default()),
            &wallpapers_dir,
            &thumbnails_cache_dir,
            &wallpapers_rescaled_dir,
        ),
        Commands::SelectWallpaper {
            socket_path,
            static_image,
        } => select_wallpaper(
            &wallpapers_dir,
            &wallpapers_rescaled_dir,
            &thumbnails_cache_dir,
            &socket_path,
            *static_image,
        ),
    }
}
