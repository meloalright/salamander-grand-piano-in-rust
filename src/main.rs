mod audio;
mod config;
mod keyboard;
mod state;
mod ui;

use std::path::PathBuf;
use std::{env, process};

use crate::audio::AudioEngine;
use crate::config::APP_CONFIG;
use crate::keyboard::{InputAction, poll_action};
use crate::state::UiState;
use crate::ui::Ui;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut ui = Ui::new()?;

    let samples_dir = resolve_samples_dir();
    let audio = AudioEngine::new(&samples_dir, APP_CONFIG.notes)?;
    let mut state = UiState::new();

    // Initial draw before entering the loop
    ui.draw(&state)?;

    loop {
        match poll_action(APP_CONFIG.notes)? {
            InputAction::Play(index, note) => {
                if let Err(err) = audio.play_note(note) {
                    eprintln!("playback error for {}: {err}", note.display);
                }
                state.note_on(index, note.display);
            }
            InputAction::Quit => break,
            InputAction::None => {}
        }

        state.set_voice_count(audio.active_voice_count());
        state.tick();
        ui.draw(&state)?;
    }

    ui.cleanup();
    Ok(())
}

fn resolve_samples_dir() -> PathBuf {
    env::var("SAMPLES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(APP_CONFIG.samples_dir))
}
