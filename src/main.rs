mod audio;
mod config;
mod keyboard;

use std::path::Path;
use std::{env, path::PathBuf};
use std::thread;
use std::time::Duration;

use crossterm::terminal;

use crate::audio::AudioEngine;
use crate::config::APP_CONFIG;
use crate::keyboard::{InputAction, poll_action};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    terminal::enable_raw_mode().map_err(|err| format!("failed to enable raw mode: {err}"))?;
    let _raw_mode_guard = RawModeGuard;

    let samples_dir = resolve_samples_dir();
    let audio = AudioEngine::new(&samples_dir, APP_CONFIG.notes)?;

    print_instructions(&samples_dir);

    loop {
        match poll_action(APP_CONFIG.notes)? {
            InputAction::Play(note) => {
                if let Err(err) = audio.play_note(note) {
                    eprintln!("playback error for {}: {err}", note.display);
                }
            }
            InputAction::Quit => break,
            InputAction::None => thread::sleep(Duration::from_millis(1)),
        }
    }

    println!("\nBye.");
    Ok(())
}

fn print_instructions(samples_dir: &Path) {
    println!("Terminal Piano");
    println!("Samples: {}", samples_dir.display());
    println!("Octave 1: A S D F G H J   -> Do Re Mi Fa So La Xi");
    println!("Octave 2: Q W E R T Y U   -> Do Re Mi Fa So La Xi");
    println!("Press Esc to quit.\n");
}

fn resolve_samples_dir() -> PathBuf {
    env::var("SAMPLES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(APP_CONFIG.samples_dir))
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
