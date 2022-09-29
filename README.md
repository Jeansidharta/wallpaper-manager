# Wallpaper Manager

This is a very simple CLI designed for my own use to manage my live wallpapers. This is only designed to work with my own system. It'll probably need some work for it to work on other systems.

## Requirements

- [ffmpeg](https://ffmpeg.org/)
- [sxiv](https://github.com/muennich/sxiv)
- [xwinwrap](https://github.com/ujjwal96/xwinwrap)
- [mpv](https://mpv.io/)

## How to install

Simply clone this project and run `cargo install --path <PATH_TO_PROJECT>`

## Some assumptions

This project assuemes your wallpapers are stored at `~/Wallpapers/live`. It's somewhat hardcoded. It also assumes `~/.cache` is your default cache directory. It also assumes `/tmp` is the system's temporary directory

## How to use

Just run the command with the `--help` option for instructions
